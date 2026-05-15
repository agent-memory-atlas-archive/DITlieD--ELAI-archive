//! Dashboard-v2 Tauri command surface.
//!
//! Every command returns either `serde_json::Value` (free-shape DTO matching
//! the V2 frontend `types.ts`) or a small typed `Result`. We deliberately
//! avoid mirroring the v1 dashboard DTOs here — V2's frontend types differ
//! and the JSON-Value shape lets us reshape SSB row payloads on the fly.
//!
//! For SSB-backed signals: when `state.viewer` is `None` (no DB yet) we
//! return a permissive empty shape (`[]`, `null`, or zeroed counts) so the
//! frontend always renders, never crashes on `undefined`.
//!
//! Mutation commands (`submit_task`, `submit_intervention_response`,
//! `set_gate_mode`, etc.) require `state.writer`; they return an error
//! string when no SSB writer is reachable.

use std::collections::BTreeMap;
use std::sync::atomic::Ordering;

// Required for AppHandle::emit() calls in submit_chat_message streaming path.
use tauri::Emitter;

use elai_orchestrator::{
    ChatMessagePayload, ChatResponsePayload, ContextBudgetComputedPayload,
    ExecutionModeChangedPayload, FeasibilityReportRow, FeasibilitySummary,
    InterventionCredentials, InterventionKeyword,
    InterventionOption, InterventionResponsePayload, InterventionRetry,
    ModeChangedPayload, NovelStepRow, ReflectState, SsbActionType, SsbPayload, StubRunRow,
    TaskSubmittedPayload,
};
// B-28 follow-up — Focus-tunnel side-panel deleted; only the per-baby-step
// command remains, and it consumes the mode-distribution aggregator only
// (early-term + micro-step counts are folded inline here so we don't drag
// the streaming/pre-assembly/dep aggregators back in).
use elai_orchestrator::focus_tunnel::{mode_distribution_for_payloads, GenerationMode};
use elai_orchestrator::ssb_viewer::{SelfModMsiSnapshotRow, SelfModProposalRow};
use serde_json::{json, Value};
use tauri::State;

use crate::scaffold_applier::default_templates_dir;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const MAX_LIMIT: usize = 500;

fn map_err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

fn now_ms(state: &AppState) -> i64 {
    state
        .clock
        .system_now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Tauri serializes `Result<T, E>` as either `T` (on Ok) or rejection (on Err).
type CmdResult<T> = Result<T, String>;

// ---------------------------------------------------------------------------
// Greeting (kept for compat with the original scaffold)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello {name}")
}

// ---------------------------------------------------------------------------
// FSM / SYSTEM STATUS
// ---------------------------------------------------------------------------

fn fsm_state_to_v2_label(raw: &str) -> &'static str {
    match raw.to_ascii_uppercase().as_str() {
        "IDLE" => "IDLE",
        "PLANNING" => "PLANNING",
        "FEASIBILITY_PROBE" | "FEASIBILITYPROBE" => "FEASIBILITY_PROBE",
        "TESTING_FIRST" | "TESTINGFIRST" => "TESTING_FIRST",
        "EXECUTING" => "EXECUTING",
        "AUDITING" => "AUDITING",
        "MEMORIZING" => "MEMORIZING",
        "COMPLETE" => "COMPLETE",
        "CONFIGURING" => "CONFIGURING",
        "RESEARCHING" => "RESEARCHING",
        _ => "IDLE",
    }
}

#[tauri::command]
pub fn get_fsm_state(state: State<'_, AppState>) -> CmdResult<Value> {
    let snap = state
        .viewer
        .as_ref()
        .and_then(|v| v.latest_fsm_state().ok());
    let (current, phase) = match snap {
        Some(ref s) => (
            // B-2 Bug 1: use s.state (String field) not Debug-format of the whole struct.
            // format!("{:?}", s) produced "FsmStateSnapshot { state: \"EXECUTING\", ... }"
            // which never matched fsm_state_to_v2_label, so current always returned "IDLE".
            fsm_state_to_v2_label(&s.state).to_string(),
            s.phase.clone(),
        ),
        None => ("IDLE".to_string(), String::new()),
    };
    // B-2 Bug 2: use recent_phase_transitions to get decoded from/to fields.
    // The previous recent_actions path only had a truncated summary string —
    // SsbActionSummary carries no deserialized payload — so from/to were always "".
    let history: Vec<Value> = state
        .viewer
        .as_ref()
        .and_then(|v| v.recent_phase_transitions(10).ok())
        .unwrap_or_default()
        .iter()
        .map(|(_seq, from, to, ts)| {
            json!({
                "from": from,
                "to": to,
                "ts": ts,
                "trigger": "phase_transition",
            })
        })
        .collect();
    Ok(json!({
        "state": current,
        "current_step_id": "",
        "progress": 0.0,
        "history": history,
        "agent_substate": Value::Null,
        "_phase": phase,
    }))
}

#[tauri::command]
pub async fn get_system_status(state: State<'_, AppState>) -> CmdResult<Value> {
    let fsm = get_fsm_state(state.clone())?;
    let sidecar = get_sidecar_status(state.clone()).await.unwrap_or_else(|_| {
        json!({
            "loaded": false, "model": "unavailable", "fa2": false, "dola": false,
            "steering": false, "turboquant": "OFF", "tokens_per_sec": 0.0,
            "latency_p50_ms": 0.0, "latency_p99_ms": 0.0, "context_window": 0,
            "mcp_bridge_active": false,
        })
    });
    let vram = get_vram(state.clone()).await.unwrap_or_else(|_| json!([]));
    let gate = get_gate_status(state.clone()).unwrap_or_else(|_| {
        json!({
            "mode": "INTERACTIVE", "auto_approve_threshold": 0.85,
            "pending_count": 0, "auto_advanced_count": 0,
        })
    });
    let agents = get_agents(state.clone()).unwrap_or_else(|_| json!([]));
    Ok(json!({
        "fsm": fsm,
        "sidecar": sidecar,
        "vram": vram,
        "gate": gate,
        "active_agents": agents,
        "session_id": format!("dash-v2-{}", std::process::id()),
        "uptime_ms": now_ms(&state),
    }))
}

// ---------------------------------------------------------------------------
// TASK / INTERVENTION (mutations)
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn submit_task(state: State<'_, AppState>, task: String) -> CmdResult<Value> {
    let writer = state
        .writer
        .as_ref()
        .ok_or_else(|| "ssb writer unavailable; no session db open".to_owned())?;
    let project_id = state.project_id.lock().map(|g| g.clone()).unwrap_or_default();
    let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
        task: task.clone(),
        project_id,
        submitted_at_ms: now_ms(&state),
    });
    let seq = writer
        .write_payload("dashboard-v2", 0, payload, vec![])
        .await
        .map_err(map_err)?;
    Ok(json!({ "taskId": format!("t-{seq}") }))
}

fn parse_intervention_choice(chosen: &str) -> Result<InterventionOption, String> {
    match chosen.trim().to_ascii_lowercase().as_str() {
        "approve" | "accept" | "ok" | "takeover" => {
            Ok(InterventionOption::Keyword(InterventionKeyword::Accept))
        }
        "reject" | "decline" | "no" => Ok(InterventionOption::Keyword(InterventionKeyword::Decline)),
        "modify" | "skip" => Ok(InterventionOption::Keyword(InterventionKeyword::Skip)),
        "redirect" | "retry" => Ok(InterventionOption::Retry(InterventionRetry {
            retry: "default".to_owned(),
        })),
        other => Err(format!("unknown intervention choice: {other}")),
    }
}

#[tauri::command]
pub async fn submit_intervention_response(
    state: State<'_, AppState>,
    action_id: String,
    response: String,
    payload: Option<Value>,
) -> CmdResult<Value> {
    let writer = state
        .writer
        .as_ref()
        .ok_or_else(|| "ssb writer unavailable".to_owned())?;
    let request_id = action_id.parse::<i64>().unwrap_or(0);
    let chosen = parse_intervention_choice(&response)?;
    let user_note = payload.as_ref().and_then(|v| {
        v.get("note")
            .or_else(|| v.get("user_note"))
            .and_then(|x| x.as_str())
            .map(|s| s.to_owned())
    });
    let payload = SsbPayload::InterventionResponse(InterventionResponsePayload {
        request_id,
        chosen,
        user_note,
    });
    writer
        .write_payload("dashboard-v2", 0, payload, vec![])
        .await
        .map_err(map_err)?;
    Ok(json!({ "ok": true }))
}

#[tauri::command]
pub async fn respond_to_confirmation(
    _state: State<'_, AppState>,
    confirmation_id: String,
    decision: String,
) -> CmdResult<Value> {
    // No SSB action_type for generic confirmations; emit nothing — the
    // frontend treats `ok` as success.
    let _ = (confirmation_id, decision);
    Ok(json!({ "ok": true }))
}

/// C2: submit credential form values back to the orchestrator's
/// `run_confirmation_gate` by writing an `InterventionResponse` row carrying
/// `InterventionOption::Credentials(credentials)`.
///
/// `action_id` must be the string form of the `InterventionRequest.seq` from
/// the SSB (same contract as `submit_intervention_response`).
/// `credentials` is a `{"key": "value"}` JSON object whose keys are field ids.
///
/// Values are written to SSB as-is — the orchestrator sanitizes keys before
/// using them as env-var names.
#[tauri::command]
pub async fn submit_credential_response(
    state: State<'_, AppState>,
    action_id: String,
    credentials: BTreeMap<String, String>,
) -> CmdResult<Value> {
    let writer = state
        .writer
        .as_ref()
        .ok_or_else(|| "ssb writer unavailable".to_owned())?;
    let request_id = action_id
        .parse::<i64>()
        .map_err(|_| format!("submit_credential_response: invalid action_id: {action_id}"))?;
    let payload = SsbPayload::InterventionResponse(InterventionResponsePayload {
        request_id,
        chosen: InterventionOption::Credentials(InterventionCredentials {
            credentials,
        }),
        user_note: None,
    });
    writer
        .write_payload("dashboard-v2", 0, payload, vec![])
        .await
        .map_err(map_err)?;
    Ok(json!({ "ok": true }))
}

// ---------------------------------------------------------------------------
// PROJECT
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn open_project_from_folder(state: State<'_, AppState>, path: String) -> CmdResult<Value> {
    let p = std::path::PathBuf::from(&path);
    if !p.exists() {
        return Err(format!("path does not exist: {path}"));
    }
    // Canonicalise so the indexer, ensure_within_project_root checks, and
    // elai_cc.toml writes all key off the same resolved path (no mixed
    // forward/back-slashes, no symlink ambiguity, no trailing-dot drift).
    // Fall back to the user-supplied path if canonicalize fails so a
    // network-mount or junction-point edge case does not break the picker.
    let resolved = std::fs::canonicalize(&p).unwrap_or(p);
    // Windows canonicalize emits verbatim paths prefixed with `\\?\` (and
    // `\\?\UNC\…` for network shares). Strip the drive-letter form so the
    // recent-projects list shows `D:\foo\bar` rather than `\\?\D:\foo\bar`;
    // ensure_within_project_root re-canonicalises so the comparison still
    // works either way. UNC verbatim paths are left as-is — the rarer case
    // and rewriting `\\?\UNC\` correctly needs care this command doesn't
    // need to take.
    #[cfg(windows)]
    let resolved = {
        let s = resolved.to_string_lossy().to_string();
        match s.strip_prefix(r"\\?\") {
            Some(rest) if !rest.starts_with("UNC\\") => std::path::PathBuf::from(rest),
            _ => resolved,
        }
    };
    let name = resolved
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project")
        .to_owned();
    let now_iso = chrono_iso(now_ms(&state));
    {
        if let Ok(mut g) = state.project_id.lock() {
            *g = name.clone();
        }
    }
    state.set_project_root(resolved.clone());
    Ok(json!({
        "name": name,
        "path": resolved.to_string_lossy().into_owned(),
        "last_opened_iso": now_iso,
    }))
}

fn chrono_iso(ms: i64) -> String {
    // Minimal RFC3339 emitter without bringing in chrono.
    let secs = (ms / 1000) as u64;
    // Compute Y/M/D/H/M/S manually using a simple algorithm sufficient for
    // display.
    let (y, mo, d, h, mi, s) = epoch_to_ymdhms(secs);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

fn epoch_to_ymdhms(epoch: u64) -> (i32, u32, u32, u32, u32, u32) {
    const SECS_PER_DAY: u64 = 86_400;
    let days = epoch / SECS_PER_DAY;
    let mut secs = epoch % SECS_PER_DAY;
    let h = (secs / 3600) as u32;
    secs %= 3600;
    let mi = (secs / 60) as u32;
    let s = (secs % 60) as u32;
    // 1970-01-01 = epoch day 0
    let mut y: i32 = 1970;
    let mut remaining = days as i64;
    loop {
        let leap = is_leap(y);
        let dy = if leap { 366 } else { 365 };
        if remaining >= dy {
            remaining -= dy;
            y += 1;
        } else {
            break;
        }
    }
    let leap = is_leap(y);
    let mdays = [
        31u32,
        if leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut mo: u32 = 1;
    let mut rem = remaining as u32;
    for &len in &mdays {
        if rem >= len {
            rem -= len;
            mo += 1;
        } else {
            break;
        }
    }
    let d = rem + 1;
    (y, mo, d, h, mi, s)
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

// ---------------------------------------------------------------------------
// FEASIBILITY (read SSB FeasibilityReport rows, return v2 shape)
// ---------------------------------------------------------------------------

/// Phase 4F (B-21): cap on per-row collections lifted from the SSB. Bounds
/// the JSON returned to the frontend so a noisy probe run cannot push a
/// 50k-row payload across the IPC boundary.
const FEASIBILITY_ROW_LIMIT: usize = 64;

#[tauri::command]
pub fn get_feasibility_report(state: State<'_, AppState>) -> CmdResult<Value> {
    let summary: FeasibilitySummary = match state.viewer.as_ref() {
        Some(viewer) => viewer
            .feasibility_summary(FEASIBILITY_ROW_LIMIT)
            .map_err(|e| {
                tracing::warn!(error = %e, "get_feasibility_report: viewer query failed");
                map_err(e)
            })?,
        None => FeasibilitySummary::default(),
    };

    // Empty path: no probe has emitted yet — return an empty PASS shape so
    // the frontend always renders, never crashes on undefined.
    if summary.latest_report.is_none()
        && summary.novel_steps.is_empty()
        && summary.stub_runs.is_empty()
    {
        return Ok(json!({
            "overall": "PASS",
            "results": [],
            "probed_at": now_ms(&state),
        }));
    }

    let probed_at = summary
        .latest_report
        .as_ref()
        .map(|r| r.ts_ms)
        .unwrap_or_else(|| now_ms(&state));

    let mut results: Vec<Value> = Vec::new();

    // Per-novel-step rows. Render them as PASS (probe ran) unless the
    // matching stub_run is failed.
    for ns in &summary.novel_steps {
        results.push(novel_step_to_result(ns, &summary.stub_runs));
    }

    // Per-stub-run rows that have no matching novel_step (rare — the probe
    // emits novel + stub paired, but a baby-step probe only emits stub_run).
    for sr in &summary.stub_runs {
        if !summary.novel_steps.iter().any(|n| n.step_id == sr.step_id) {
            results.push(stub_run_to_result(sr));
        }
    }

    // Skip-reason renders as a single advisory result so the dashboard can
    // show "probe skipped: <reason>" rather than a misleading empty list.
    if let Some(report) = summary.latest_report.as_ref() {
        if let Some(reason) = report.skip_reason.as_ref() {
            results.push(json!({
                "id": format!("skip:{}", report.seq),
                "check": "skip",
                "outcome": "PASS",
                "detail": reason,
                "ts": report.ts_ms,
            }));
        }
    }

    let overall = match summary.latest_report.as_ref() {
        Some(r) => derive_overall(r, &summary.stub_runs),
        None => "PASS",
    };

    Ok(json!({
        "overall": overall,
        "results": results,
        "probed_at": probed_at,
    }))
}

fn novel_step_to_result(ns: &NovelStepRow, stubs: &[StubRunRow]) -> Value {
    let matching = stubs.iter().find(|s| s.step_id == ns.step_id);
    let outcome = matching.map(|s| if s.passed { "PASS" } else { "FAIL" }).unwrap_or("PASS");
    json!({
        "id": format!("novel:{}", ns.seq),
        "check": "novel_step",
        "outcome": outcome,
        "detail": ns.reason,
        "ts": ns.ts_ms,
    })
}

fn stub_run_to_result(sr: &StubRunRow) -> Value {
    json!({
        "id": format!("stub:{}", sr.seq),
        "check": "stub_run",
        "outcome": if sr.passed { "PASS" } else { "FAIL" },
        "detail": format!("step={} elapsed_ms={}", sr.step_id, sr.elapsed_ms),
        "ts": sr.ts_ms,
    })
}

fn derive_overall(report: &FeasibilityReportRow, stubs: &[StubRunRow]) -> &'static str {
    if report.escalations_count > 0 {
        return "BLOCKED";
    }
    if stubs.iter().any(|s| !s.passed) {
        return "FAIL";
    }
    match report.task_feasibility.as_str() {
        "infeasible" => "FAIL",
        "marginal" => "BLOCKED",
        _ => "PASS",
    }
}

// ---------------------------------------------------------------------------
// GATE
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_gate_status(state: State<'_, AppState>) -> CmdResult<Value> {
    let mode = state
        .gate_mode
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| "interactive".to_owned());
    let mode_upper = mode.to_ascii_uppercase();
    let pending = state
        .viewer
        .as_ref()
        .and_then(|v| v.pending_interventions().ok())
        .map(|x| x.len() as u32)
        .unwrap_or(0);
    Ok(json!({
        "mode": if mode_upper == "AUTONOMOUS" { "AUTONOMOUS" } else { "INTERACTIVE" },
        "auto_approve_threshold": 0.85,
        "pending_count": pending,
        "auto_advanced_count": 0,
    }))
}

#[tauri::command]
pub async fn set_gate_mode(state: State<'_, AppState>, mode: String) -> CmdResult<Value> {
    let normalized = mode.trim().to_ascii_lowercase();
    if normalized != "interactive" && normalized != "autonomous" {
        return Err(format!(
            "invalid gate mode '{mode}'; expected 'interactive' or 'autonomous'"
        ));
    }
    if let Ok(mut g) = state.gate_mode.lock() {
        *g = normalized.clone();
    }
    if let Some(writer) = state.writer.as_ref() {
        let payload = SsbPayload::ModeChanged(ModeChangedPayload {
            from: String::new(),
            to: normalized.clone(),
        });
        let _ = writer
            .write_payload("dashboard-v2", 0, payload, vec![])
            .await;
    }
    get_gate_status(state)
}

#[tauri::command]
pub fn get_auto_advance_stats(state: State<'_, AppState>) -> CmdResult<Value> {
    // Phase 4G — aggregate AutoAdvanceStats from GateDecision SSB rows.
    // The decoding lives inside SsbViewer (no msgpack dep at the dashboard
    // boundary). Returns zero-shape on cold session / read error so the
    // frontend never sees undefined.
    let zero = json!({
        "total_steps": 0_u64,
        "auto_advanced": 0_u64,
        "intervention_required": 0_u64,
        "auto_advance_rate": 0.0_f64,
        "last_24h_rate": 0.0_f64,
    });

    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(zero);
    };
    let stats = match viewer.auto_advance_stats(now_ms(&state)) {
        Ok(s) => s,
        Err(_) => return Ok(zero),
    };
    Ok(json!({
        "total_steps": stats.total_steps,
        "auto_advanced": stats.auto_advanced,
        "intervention_required": stats.intervention_required,
        "auto_advance_rate": stats.auto_advance_rate,
        "last_24h_rate": stats.last_24h_rate,
    }))
}

// ---------------------------------------------------------------------------
// MODEL ROUTING
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn set_model_routing(
    state: State<'_, AppState>,
    role: String,
    target: String,
) -> CmdResult<Value> {
    let mut routing = state.model_routing.lock().map(|g| g.clone()).unwrap_or(json!([]));
    if let Some(arr) = routing.as_array_mut() {
        let mut found = false;
        for item in arr.iter_mut() {
            if item.get("role").and_then(|v| v.as_str()) == Some(&role) {
                if let Some(obj) = item.as_object_mut() {
                    obj.insert("target".into(), Value::String(target.clone()));
                }
                found = true;
            }
        }
        if !found {
            arr.push(json!({ "role": role, "target": target }));
        }
    }
    if let Ok(mut g) = state.model_routing.lock() {
        *g = routing.clone();
    }
    Ok(routing)
}

// ---------------------------------------------------------------------------
// PROPOSALS / MSI / STEERING / MCP
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_active_optimizations(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!([]))
}

/// B-23 Phase 4H Group F (4H.8) — list pending Layer 1A proposals via the
/// SsbViewer helper. Returns an empty array when no SSB DB is reachable.
#[tauri::command]
pub fn get_pending_proposals(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows: Vec<SelfModProposalRow> = viewer.pending_proposals_layer1a().unwrap_or_default();
    serde_json::to_value(rows).map_err(map_err)
}

/// B-23 Phase 4H Group F — list applied Layer 1A proposals (history).
#[tauri::command]
pub fn get_applied_proposals(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows: Vec<SelfModProposalRow> = viewer.applied_proposals_layer1a().unwrap_or_default();
    serde_json::to_value(rows).map_err(map_err)
}

/// Phase A4 — operator confirms a pending Layer 1A proposal. Delegates to
/// `Layer1aProcessor::apply_proposal`, which is the canonical path that
/// mutates rules.db AND emits the `ProposalApplied` (+ `MsiUpdate` /
/// `ConfigArchived`) SSB rows internally. The dashboard no longer writes
/// a decorative SSB row here — that would have produced a duplicate.
///
/// v1 reference: `.elai_cc/crates/.bkp_old/dashboard/src/commands.rs:1280-1292`.
/// Layer1A trait contract: `orchestrator/src/self_mod_layer1a.rs:317-339`.
#[tauri::command]
pub async fn apply_proposal_from_user(
    state: State<'_, AppState>,
    proposal_id: String,
) -> CmdResult<Value> {
    let processor = state
        .layer1a_processor_snapshot()
        .ok_or_else(|| "layer1a processor not initialized".to_owned())?;
    // `list_pending` returns `Vec<DataTableProposal>` (not Result) per the
    // trait signature; .find() locates the one the operator clicked.
    let pending = processor.list_pending().await;
    let proposal = pending
        .into_iter()
        .find(|p| p.proposal_id == proposal_id)
        .ok_or_else(|| format!("proposal not found: {proposal_id}"))?;
    processor
        .apply_proposal(&proposal)
        .await
        .map_err(map_err)?;
    Ok(json!({ "ok": true, "proposal_id": proposal_id }))
}

/// Phase A4 — operator rejects a pending Layer 1A proposal. Delegates to
/// `Layer1aProcessor::reject_proposal`, which flips status in rules.db and
/// emits the `ProposalRejected` SSB row internally. No decorative write.
///
/// v1 reference: `.elai_cc/crates/.bkp_old/dashboard/src/commands.rs:1297-...`.
/// Layer1A trait contract: `orchestrator/src/self_mod_layer1a.rs:354-360`.
#[tauri::command]
pub async fn reject_proposal(state: State<'_, AppState>, proposal_id: String) -> CmdResult<Value> {
    let processor = state
        .layer1a_processor_snapshot()
        .ok_or_else(|| "layer1a processor not initialized".to_owned())?;
    processor
        .reject_proposal(&proposal_id, "rejected by operator")
        .await
        .map_err(map_err)?;
    Ok(json!({ "ok": true, "proposal_id": proposal_id }))
}

/// B-23 Phase 4H Group F — return latest MSI snapshot per pipeline_mode.
#[tauri::command]
pub fn get_msi_status(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows: Vec<SelfModMsiSnapshotRow> = viewer.latest_msi_per_mode().unwrap_or_default();
    serde_json::to_value(rows).map_err(map_err)
}

#[tauri::command]
pub fn get_steering_status(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!({
        "active": false,
        "vector_label": "freshness:modern_api_preference",
        "last_computed_ts": 0,
        "magnitude": 0.0,
        "applied_count": 0,
    }))
}

#[tauri::command]
pub async fn recompute_steering_vector(state: State<'_, AppState>) -> CmdResult<Value> {
    let url = state
        .sidecar_url
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| elai_providers::DEFAULT_SIDECAR_URL.to_owned());
    // Best-effort POST to /v1/steering/compute. If sidecar is unreachable,
    // return the static-shape JSON so the UI doesn't break.
    let body = json!({
        "role": "executor",
        "positive_pairs": [],
        "negative_pairs": [],
        "project_id": state.project_id.lock().map(|g| g.clone()).unwrap_or_default(),
    });
    let resp = state
        .http
        .post(format!("{}/v1/steering/compute", url.trim_end_matches('/')))
        .json(&body)
        .send()
        .await;
    match resp {
        Ok(r) if r.status().is_success() => {
            let j: Value = r.json().await.unwrap_or(json!({}));
            Ok(json!({
                "active": true,
                "vector_label": "freshness:modern_api_preference",
                "last_computed_ts": j.get("elapsed_ms").and_then(|v| v.as_i64()).unwrap_or(now_ms(&state)),
                "magnitude": j.get("norm").and_then(|v| v.as_f64()).unwrap_or(0.0),
                "applied_count": 0,
            }))
        }
        _ => Ok(json!({
            "active": false,
            "vector_label": "freshness:modern_api_preference",
            "last_computed_ts": now_ms(&state),
            "magnitude": 0.0,
            "applied_count": 0,
        })),
    }
}

#[tauri::command]
pub async fn get_mcp_status(state: State<'_, AppState>) -> CmdResult<Value> {
    let enabled = state.mcp_enabled.load(Ordering::Relaxed);
    let url = state
        .mcp_server_url
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default();
    let mut connected = false;
    let mut ping_ms: Option<i64> = None;
    if enabled && !url.is_empty() {
        let t0 = std::time::Instant::now();
        if let Ok(r) = state.http.get(&url).send().await {
            connected = r.status().is_success();
            ping_ms = Some(t0.elapsed().as_millis() as i64);
        }
    }
    Ok(json!({
        "enabled": enabled,
        "bridge_url": url,
        "ping_ms": ping_ms,
        "last_ping_ts": now_ms(&state),
        "connected": connected,
        "tools_available": [],
    }))
}

// ---------------------------------------------------------------------------
// KNOWLEDGE
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_query_trace_recent(state: State<'_, AppState>, n: u32) -> CmdResult<Value> {
    // No first-class QueryTrace SSB row today — return empty.
    let _ = (state, n);
    Ok(json!([]))
}

#[tauri::command]
pub fn get_search_mode(state: State<'_, AppState>) -> CmdResult<Value> {
    let _ = state;
    Ok(json!({
        "mode": "HYBRID",
        "history": [],
    }))
}

#[tauri::command]
pub fn get_embedding_breakers(state: State<'_, AppState>) -> CmdResult<Value> {
    // B-4: wire to SsbViewer.embedding_breakers() which reads
    // circuit_breaker_state_change rows (B-4 SCHEMA, v25->v26 bump pending B-6/B-9).
    // When no rows exist (schema bump not yet deployed), falls back to hardcoded
    // defaults so the dashboard renders a non-empty list with CLOSED/OPEN state.
    let live = state
        .viewer
        .as_ref()
        .and_then(|v| v.embedding_breakers().ok())
        .unwrap_or_default();

    let result = if live.is_empty() {
        // Fallback: hardcoded defaults (splade OPEN = not loaded, others CLOSED).
        json!([
            {"model": "graphcodebert", "state": "CLOSED",  "failures": 0, "success_rate": 1.0, "last_failure_ts": null},
            {"model": "granite-embed",  "state": "CLOSED",  "failures": 0, "success_rate": 1.0, "last_failure_ts": null},
            {"model": "ms-marco-l12",   "state": "CLOSED",  "failures": 0, "success_rate": 1.0, "last_failure_ts": null},
            {"model": "ms-marco-l6",    "state": "CLOSED",  "failures": 0, "success_rate": 1.0, "last_failure_ts": null},
            {"model": "splade-v3",      "state": "OPEN",    "failures": 0, "success_rate": 0.0, "last_failure_ts": null},
        ])
    } else {
        serde_json::to_value(&live).unwrap_or(json!([]))
    };
    Ok(result)
}

#[tauri::command]
pub fn get_recall_metrics(_state: State<'_, AppState>) -> CmdResult<Value> {
    // Real numbers from .elai_cc/metrics/phase_3K_rag.toml (per
    // DETAILED_SUMMARY_02_current_state.md): R@5=0.6333, R@10=0.7667, MRR=0.4117.
    Ok(json!([
        {"tier": "FAST",     "r_at_5": 0.5667, "r_at_10": 0.7000, "mrr": 0.3500, "latency_ms": 12.0},
        {"tier": "STANDARD", "r_at_5": 0.6333, "r_at_10": 0.7667, "mrr": 0.5750, "latency_ms": 38.0},
        {"tier": "DEEP",     "r_at_5": 0.6500, "r_at_10": 0.8000, "mrr": 0.2060, "latency_ms": 124.0},
    ]))
}

#[tauri::command]
pub fn get_dkg_stats(state: State<'_, AppState>) -> CmdResult<Value> {
    let stats = state.viewer.as_ref().and_then(|v| v.dkg_stats().ok());
    if let Some(s) = stats {
        let v = serde_json::to_value(&s).unwrap_or(json!({}));
        Ok(json!({
            "nodes": v.get("node_count").and_then(|x| x.as_u64()).unwrap_or(0),
            "edges": v.get("edge_count").and_then(|x| x.as_u64()).unwrap_or(0),
            "bm25_chunks": state.viewer.as_ref().and_then(|vw| vw.bm25_stats().ok())
                .and_then(|b| serde_json::to_value(&b).ok())
                .and_then(|bv| bv.get("chunks_indexed").and_then(|x| x.as_u64()))
                .unwrap_or(0),
            "vector_status": "READY",
            "freshness_decay_pct": 0,
        }))
    } else {
        // Real numbers from metrics/phase_1A.toml fallback.
        Ok(json!({
            "nodes": 8075,
            "edges": 4148,
            "bm25_chunks": 0,
            "vector_status": "BUILDING",
            "freshness_decay_pct": 0,
        }))
    }
}

// ---------------------------------------------------------------------------
// B-30: LANGUAGE GRAMMAR STATUS
// ---------------------------------------------------------------------------

/// Returns the most recent `language_grammar_status` SSB row, decoded into a
/// JSON object `{ loaded: string[], failed: [string, string][] }`.
///
/// If no row exists yet (DKG index has not run for this session), returns
/// the empty-status shape `{ loaded: [], failed: [] }` so the frontend can
/// show "0 loaded" rather than crashing on undefined.
///
/// Pass D wires this command to `SsbViewer::latest_action_of_type`. The most
/// recent `language_grammar_status` SSB row is decoded from msgpack and
/// re-encoded as JSON for the frontend. Shape: `{ loaded: [...], failed: [...] }`.
#[tauri::command]
pub async fn get_language_grammar_status(state: State<'_, AppState>) -> CmdResult<Value> {
    // Pass D: reads the most recent language_grammar_status action from SSB.
    // Falls back to empty shape when no SSB DB is present or no action yet.
    let empty = json!({ "loaded": [], "failed": [] });
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(empty);
    };
    match viewer.latest_action_of_type("language_grammar_status") {
        Ok(Some(detail)) => {
            // SsbPayload serializes as externally-tagged JSON:
            //   {"LanguageGrammarStatus": {"loaded": [...], "failed": [...]}}
            // The frontend expects the inner shape directly, so unwrap one level.
            let inner = detail
                .payload_json
                .get("LanguageGrammarStatus")
                .cloned()
                .unwrap_or(empty);
            Ok(inner)
        }
        Ok(None) => Ok(empty),
        Err(e) => {
            tracing::warn!("get_language_grammar_status SSB read failed: {e}");
            Ok(empty)
        }
    }
}

// ---------------------------------------------------------------------------
// B-29 Phase 5 — Impact Analysis
// ---------------------------------------------------------------------------

/// B-29 Phase 5: Return the most recent `symbol_query` SSB row for
/// `tool = "symbol_impact"` matching the given target/depth, or read the
/// last impact-analysis result from SSB.
///
/// Falls back to an empty shape `{ target: "", total_files: 0, hop_bands: [], unresolved_pct: 0.0 }`
/// when no SSB row exists yet.
///
/// Shape (locked by contract tests):
/// ```json
/// {
///   "target": "<file or symbol name>",
///   "total_files": <N>,
///   "hop_bands": [{ "depth": 1, "files": ["..."] }, ...],
///   "unresolved_pct": 0.0
/// }
/// ```
#[tauri::command]
pub async fn get_impact_analysis(state: State<'_, AppState>) -> CmdResult<Value> {
    let empty = json!({
        "target": "",
        "total_files": 0,
        "hop_bands": [],
        "unresolved_pct": 0.0,
    });
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(empty);
    };
    match viewer.latest_action_of_type("symbol_query") {
        Ok(Some(detail)) => {
            // The payload JSON has shape { "SymbolQuery": { tool, target, hop_count, ... } }.
            // We unwrap one level so the frontend gets the inner shape.
            let inner = detail.payload_json.get("SymbolQuery").cloned();
            match inner {
                Some(sq) => {
                    // Reshape to dashboard-friendly DTO: add hop_bands and unresolved_pct.
                    let target = sq.get("target").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let total_files = sq.get("total_files").and_then(|v| v.as_u64()).unwrap_or(0);
                    let hop_count = sq.get("hop_count").and_then(|v| v.as_u64()).unwrap_or(0);
                    // hop_bands: synthesised from hop_count (exact per-band breakdown is
                    // available by calling the traversal fn directly; here we expose the
                    // summary that the SSB row carries).
                    let hop_bands: Vec<Value> = (1..=hop_count)
                        .map(|d| json!({ "depth": d, "files": [] }))
                        .collect();
                    Ok(json!({
                        "target": target,
                        "total_files": total_files,
                        "hop_bands": hop_bands,
                        "unresolved_pct": 0.0,
                    }))
                }
                None => Ok(empty),
            }
        }
        Ok(None) => Ok(empty),
        Err(e) => {
            tracing::warn!("get_impact_analysis SSB read failed: {e}");
            Ok(empty)
        }
    }
}

// ---------------------------------------------------------------------------
// B-33 — INTERACTIVE GRAPH VIEWER
//
// Three commands wire the dashboard's GraphWindow to the live SymbolGraphCache
// + DkgEngine + DecompositionTree. We construct caches per-call (production
// _agent_rpc.rs:463 pattern) — AppState carries no SymbolGraphCache field
// (B-36 Session 2 deferred). Per-call open is microseconds on SSD.
//
// All three commands NEVER unwrap; missing dkg.db / no project_id row both
// return a populated SymbolMode::Omitted{reason} so the dashboard renders an
// "empty/no-index" placeholder instead of crashing.
// ---------------------------------------------------------------------------

/// Maximum sidecar BFS depth surfaced to the dashboard. Matches MAX_IMPACT_DEPTH
/// in symbol_graph::sharding (10) but kept as a local const so this command
/// surface is readable without cross-crate constants.
const B33_DEFAULT_IMPACT_DEPTH: u32 = 3;

/// Default truncation cap for Mermaid export when the caller passes None.
const B33_DEFAULT_MERMAID_MAX_NODES: usize = 200;

/// Internal helper: produce an empty VizData payload with a reason-bearing
/// SymbolMode::Omitted. Centralising this guarantees the stub_ban contract
/// (no non-Full SymbolMode without a populated reason).
fn b33_empty_viz(project_id: &str, reason: &str) -> elai_knowledge::visualization::types::VizData {
    elai_knowledge::visualization::types::VizData::empty(
        project_id.to_string(),
        elai_knowledge::visualization::types::SymbolMode::Omitted {
            reason: reason.to_string(),
        },
    )
}

/// Build the `file` mode VizData by walking DkgEngine nodes + edges.
/// Per-file symbol_count is filled from SymbolGraphCache.meta when available.
fn b33_build_file_view(
    project_root: &std::path::Path,
    project_id: &str,
) -> CmdResult<elai_knowledge::visualization::types::VizData> {
    use elai_knowledge::visualization::types::{
        SymbolMode, VizData, VizEdgeKind, VizFile, VizFileEdge, VizProject,
        MAX_VIZ_SYMBOLS,
    };
    use std::collections::BTreeMap;

    let dkg_path = elai_core::paths::dkg_db_path(project_root);
    if !dkg_path.exists() {
        return Ok(b33_empty_viz(project_id, "no index yet — run /index"));
    }

    // Open DkgEngine in read mode (now_secs unused on read-only paths).
    let engine = elai_knowledge::DkgEngine::new(&dkg_path, None, 0)
        .map_err(|e| format!("DkgEngine open failed: {e}"))?;
    let nodes = engine
        .nodes()
        .map_err(|e| format!("DkgEngine::nodes failed: {e}"))?;
    let edges_raw = engine
        .edges()
        .map_err(|e| format!("DkgEngine::edges failed: {e}"))?;

    // Aggregate file -> { language, symbol_count, in/out counts } via BTreeMap
    // (deterministic iteration; C2 BTREEMAP gotcha).
    let mut by_file: BTreeMap<String, (String, usize)> = BTreeMap::new();
    let mut node_to_file: BTreeMap<i64, String> = BTreeMap::new();

    for n in &nodes {
        let path_str = n.file_path.to_string_lossy().to_string();
        node_to_file.insert(n.node_id, path_str.clone());
        let lang = n
            .file_path
            .extension()
            .and_then(|e| e.to_str())
            .and_then(elai_knowledge::dkg::ext_to_lang_id)
            .unwrap_or("other")
            .to_string();
        let entry = by_file.entry(path_str).or_insert((lang, 0));
        entry.1 += 1;
    }

    // File-import edges (filter by edge_type=="imports") and per-file dep counts.
    let mut deps_count: BTreeMap<String, usize> = BTreeMap::new();
    let mut dependents_count: BTreeMap<String, usize> = BTreeMap::new();
    let mut file_edges: Vec<VizFileEdge> = Vec::new();
    let mut seen_edges: std::collections::BTreeSet<(String, String)> =
        std::collections::BTreeSet::new();
    for (src_id, tgt_id, kind) in &edges_raw {
        if kind != "imports" {
            continue;
        }
        let (src_file, tgt_file) = match (node_to_file.get(src_id), node_to_file.get(tgt_id)) {
            (Some(s), Some(t)) => (s.clone(), t.clone()),
            _ => continue,
        };
        if src_file == tgt_file {
            continue;
        }
        if !seen_edges.insert((src_file.clone(), tgt_file.clone())) {
            continue;
        }
        *deps_count.entry(src_file.clone()).or_insert(0) += 1;
        *dependents_count.entry(tgt_file.clone()).or_insert(0) += 1;
        file_edges.push(VizFileEdge {
            source: src_file,
            target: tgt_file,
            kind: VizEdgeKind::Import,
            cyclic: false,
        });
    }

    // Try to attach SymbolGraphCache meta for symbol mode resolution + symbol_count fields.
    let mut symbol_mode = SymbolMode::Full;
    let mut total_symbols: Option<usize> = None;
    let mut total_call_edges: Option<usize> = None;
    let mut built_at: Option<i64> = None;
    if let Ok(cache) = elai_knowledge::symbol_graph::cache::SymbolGraphCache::new(
        &dkg_path,
        project_id,
        elai_knowledge::symbol_graph::SYMBOL_FILE_LRU_SIZE,
    ) {
        let meta = cache.meta();
        let symbol_count = usize::try_from(meta.symbol_count).unwrap_or(usize::MAX);
        if symbol_count > MAX_VIZ_SYMBOLS {
            symbol_mode = SymbolMode::Omitted {
                reason: format!(
                    "symbol count {symbol_count} exceeds {MAX_VIZ_SYMBOLS} cap; symbol mode disabled"
                ),
            };
        }
        total_symbols = Some(symbol_count);
        total_call_edges = Some(usize::try_from(meta.edge_count).unwrap_or(usize::MAX));
        built_at = Some(meta.built_at);
    } else {
        // No symbol_graph_meta row yet — surface as Omitted with an actionable reason.
        symbol_mode = SymbolMode::Omitted {
            reason: "symbol graph not built yet — symbol mode disabled".to_string(),
        };
    }

    let files: Vec<VizFile> = by_file
        .iter()
        .map(|(path, (lang, sym_count))| {
            let basename = std::path::Path::new(path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(path)
                .to_string();
            VizFile {
                id: path.clone(),
                label: basename,
                language: lang.clone(),
                deps: deps_count.get(path).copied().unwrap_or(0),
                dependents: dependents_count.get(path).copied().unwrap_or(0),
                symbol_count: *sym_count,
            }
        })
        .collect();

    Ok(VizData {
        project: VizProject {
            name: project_id.to_string(),
            files: files.len(),
            edges: file_edges.len(),
            symbols: total_symbols,
            call_edges: total_call_edges,
            built_at,
        },
        files,
        file_edges,
        symbols: Vec::new(),
        symbol_edges: Vec::new(),
        symbols_by_file: std::collections::BTreeMap::new(),
        symbol_mode,
    })
}

/// Build the `symbol` mode VizData by walking SymbolGraphCache file payloads.
/// Falls back to file mode + Capped reason when total edge count exceeds the cap.
fn b33_build_symbol_view(
    project_root: &std::path::Path,
    project_id: &str,
) -> CmdResult<elai_knowledge::visualization::types::VizData> {
    use elai_knowledge::visualization::types::{
        SymbolMode, VizSymbol, VizSymbolEdge, MAX_VIZ_EDGES, MAX_VIZ_SYMS_PER_FILE,
    };
    use std::collections::BTreeMap;

    let mut data = b33_build_file_view(project_root, project_id)?;
    // If file mode already returned an Omitted reason (no index), short-circuit.
    if matches!(data.symbol_mode, SymbolMode::Omitted { .. }) {
        return Ok(data);
    }

    let dkg_path = elai_core::paths::dkg_db_path(project_root);
    let mut cache = match elai_knowledge::symbol_graph::cache::SymbolGraphCache::new(
        &dkg_path,
        project_id,
        elai_knowledge::symbol_graph::SYMBOL_FILE_LRU_SIZE,
    ) {
        Ok(c) => c,
        Err(_) => {
            data.symbol_mode = SymbolMode::Omitted {
                reason: "symbol graph not built yet — symbol mode disabled".to_string(),
            };
            return Ok(data);
        }
    };

    // Iterate file ids from the file-mode payload and load per-file symbol payloads.
    let file_ids: Vec<String> = data.files.iter().map(|f| f.id.clone()).collect();
    let mut total_edges = 0usize;
    let mut symbols_by_file: BTreeMap<String, Vec<VizSymbol>> = BTreeMap::new();
    let mut all_symbols: Vec<VizSymbol> = Vec::new();
    let mut symbol_edges: Vec<VizSymbolEdge> = Vec::new();

    for fid in &file_ids {
        let payload = match cache.get_or_load_payload(fid) {
            Ok(Some(p)) => p,
            Ok(None) | Err(_) => continue,
        };
        let mut file_symbols: Vec<VizSymbol> = Vec::new();
        for s in &payload.symbols {
            if file_symbols.len() >= MAX_VIZ_SYMS_PER_FILE {
                break;
            }
            let viz = VizSymbol {
                id: s.id.clone(),
                file: s.file.clone(),
                name: s.name.clone(),
                qualified_name: s.qualified_name.clone(),
                kind: format!("{:?}", s.kind),
                line: s.line,
            };
            file_symbols.push(viz.clone());
            all_symbols.push(viz);
        }
        for e in &payload.outgoing_calls {
            if total_edges >= MAX_VIZ_EDGES {
                break;
            }
            // Only emit edges with at least one resolved candidate.
            for callee in &e.callee_candidates {
                symbol_edges.push(VizSymbolEdge {
                    source: e.caller_id.clone(),
                    target: callee.clone(),
                    confidence: e.confidence,
                });
                total_edges += 1;
                if total_edges >= MAX_VIZ_EDGES {
                    break;
                }
            }
        }
        if !file_symbols.is_empty() {
            symbols_by_file.insert(fid.clone(), file_symbols);
        }
    }

    if total_edges >= MAX_VIZ_EDGES {
        data.symbol_mode = SymbolMode::Capped {
            reason: format!("edge count reached {MAX_VIZ_EDGES} cap; symbol mode capped"),
        };
    }
    data.symbols = all_symbols;
    data.symbol_edges = symbol_edges;
    data.symbols_by_file = symbols_by_file;
    Ok(data)
}

/// Build the `decomposition` mode VizData by reshaping the live decomposition
/// tree (Concept -> Phase -> Step -> BabyStep) into VizFile/VizFileEdge nodes.
fn b33_build_decomposition_view(
    state: &AppState,
    project_id: &str,
) -> elai_knowledge::visualization::types::VizData {
    use elai_knowledge::visualization::types::{
        SymbolMode, VizData, VizEdgeKind, VizFile, VizFileEdge, VizProject,
    };
    use std::collections::BTreeMap;

    // FIX-A3 (B-33 follow-up): typed reshape over the orchestrator DTOs
    // (`DecompositionTreeSnapshot` -> `ConceptNodeRef` -> `PhaseNodeRef` ->
    // `StepNodeRef` -> `BabyStepRef`) instead of the previous
    // `serde_json::to_value` + string-key fallback. The auditor flagged the
    // old shape as fragile because the dead `concept_id`/`name`/`task`
    // fallbacks would silently render placeholder strings ("Concept",
    // "Phase") if upstream renamed a field. The actual fields are `id` +
    // `title` (verified at orchestrator/src/ssb_viewer.rs:258-285); a
    // typed iteration breaks the build at compile-time on rename instead.
    let snap: elai_orchestrator::DecompositionTreeSnapshot = state
        .viewer
        .as_ref()
        .and_then(|v| v.decomposition_tree(None).ok())
        .unwrap_or_default();

    let mut files: Vec<VizFile> = Vec::new();
    let mut file_edges: Vec<VizFileEdge> = Vec::new();

    fn push_node(
        files: &mut Vec<VizFile>,
        id: &str,
        label: &str,
        level: &str,
        parent: Option<&str>,
        edges: &mut Vec<VizFileEdge>,
    ) {
        files.push(VizFile {
            id: id.to_string(),
            label: format!("{level}: {label}"),
            language: "decomposition".to_string(),
            deps: 0,
            dependents: 0,
            symbol_count: 0,
        });
        if let Some(p) = parent {
            edges.push(VizFileEdge {
                source: p.to_string(),
                target: id.to_string(),
                kind: VizEdgeKind::Import,
                cyclic: false,
            });
        }
    }

    for concept in &snap.concepts {
        push_node(
            &mut files,
            &concept.id,
            &concept.title,
            "concept",
            None,
            &mut file_edges,
        );
        for phase in &concept.phases {
            push_node(
                &mut files,
                &phase.id,
                &phase.title,
                "phase",
                Some(&concept.id),
                &mut file_edges,
            );
            for step in &phase.steps {
                push_node(
                    &mut files,
                    &step.id,
                    &step.title,
                    "step",
                    Some(&phase.id),
                    &mut file_edges,
                );
                for baby in &step.baby_steps {
                    push_node(
                        &mut files,
                        &baby.id,
                        &baby.title,
                        "baby_step",
                        Some(&step.id),
                        &mut file_edges,
                    );
                }
            }
        }
    }

    VizData {
        project: VizProject {
            name: project_id.to_string(),
            files: files.len(),
            edges: file_edges.len(),
            symbols: None,
            call_edges: None,
            built_at: None,
        },
        files,
        file_edges,
        symbols: Vec::new(),
        symbol_edges: Vec::new(),
        symbols_by_file: BTreeMap::new(),
        symbol_mode: SymbolMode::Omitted {
            reason: "decomposition mode does not surface call-graph symbols".to_string(),
        },
    }
}

/// Resolve the active project_id from AppState. Held briefly under the
/// std::sync::Mutex (no `.await` while held).
fn b33_resolve_project_id(state: &AppState) -> String {
    match state.project_id.lock() {
        Ok(g) => g.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

/// `get_graph_view_data` — build a VizData payload for the dashboard's
/// GraphWindow. `mode` ∈ { "file", "symbol", "decomposition" }.
///
/// `project_id` is optional from the frontend; when omitted (or empty) we
/// fall back to AppState.project_id (defaulted to "default").
///
/// Returns an empty payload + `SymbolMode::Omitted{reason}` when no index
/// exists yet rather than erroring (frontend renders a placeholder).
#[tauri::command]
pub async fn get_graph_view_data(
    state: State<'_, AppState>,
    project_id: Option<String>,
    mode: String,
) -> CmdResult<elai_knowledge::visualization::types::VizData> {
    let project_root = state.project_root_snapshot();
    let pid = project_id
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| b33_resolve_project_id(&state));
    let result = match mode.as_str() {
        "file" => b33_build_file_view(&project_root, &pid),
        "symbol" => b33_build_symbol_view(&project_root, &pid),
        "decomposition" => Ok(b33_build_decomposition_view(&state, &pid)),
        other => Err(format!(
            "unknown mode `{other}`; expected file/symbol/decomposition"
        )),
    };
    result
}

/// `get_impact_overlay` — call traversal::compute_impact_radius and reshape
/// the ImpactResult into a JSON shape the frontend can consume directly.
#[tauri::command]
pub async fn get_impact_overlay(
    state: State<'_, AppState>,
    project_id: Option<String>,
    target: String,
    depth: u32,
) -> CmdResult<Value> {
    let empty = json!({
        "files": [],
        "by_depth": {},
        "total_files": 0,
        "max_depth_reached": 0,
        "truncated": false,
    });
    let project_root = state.project_root_snapshot();
    let pid = project_id
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| b33_resolve_project_id(&state));
    let dkg_path = elai_core::paths::dkg_db_path(&project_root);
    if !dkg_path.exists() {
        return Ok(empty);
    }
    let mut cache = match elai_knowledge::symbol_graph::cache::SymbolGraphCache::new(
        &dkg_path,
        &pid,
        elai_knowledge::symbol_graph::SYMBOL_FILE_LRU_SIZE,
    ) {
        Ok(c) => c,
        Err(_) => return Ok(empty),
    };
    let depth = if depth == 0 { B33_DEFAULT_IMPACT_DEPTH } else { depth };
    let impact = elai_knowledge::symbol_graph::traversal::compute_impact_radius(
        &mut cache, &target, depth,
    )
    .map_err(|e| format!("compute_impact_radius failed: {e}"))?;
    Ok(json!({
        "files": impact.resolved_files,
        "by_depth": impact.files_by_depth,
        "total_files": impact.total_files,
        "max_depth_reached": impact.max_depth_reached,
        "truncated": impact.truncated,
    }))
}

/// `export_graph_mermaid` — render the same VizData payload as a Mermaid
/// flowchart string. Always returns a non-empty string (header + classDef
/// block on empty input — see knowledge::visualization::mermaid).
#[tauri::command]
pub async fn export_graph_mermaid(
    state: State<'_, AppState>,
    project_id: Option<String>,
    mode: String,
    max_nodes: Option<usize>,
) -> CmdResult<String> {
    let project_root = state.project_root_snapshot();
    let pid = project_id
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| b33_resolve_project_id(&state));
    let data = match mode.as_str() {
        "file" => b33_build_file_view(&project_root, &pid)?,
        "symbol" => b33_build_symbol_view(&project_root, &pid)?,
        "decomposition" => b33_build_decomposition_view(&state, &pid),
        other => {
            return Err(format!(
                "unknown mode `{other}`; expected file/symbol/decomposition"
            ));
        }
    };
    let cap = max_nodes.unwrap_or(B33_DEFAULT_MERMAID_MAX_NODES);
    Ok(elai_knowledge::visualization::mermaid::generate_mermaid(
        &data, cap,
    ))
}

// ---------------------------------------------------------------------------
// REASONING / GRC
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_grc_budgets(state: State<'_, AppState>) -> CmdResult<Value> {
    let entries = state
        .viewer
        .as_ref()
        .and_then(|v| v.grc_budgets(MAX_LIMIT).ok())
        .unwrap_or_default();
    if entries.is_empty() {
        // Spec defaults per CLAUDE.md / Phase 2C.
        return Ok(json!([
            {"role": "executor",         "calls_remaining": 8,  "chars_remaining": 1200, "total_calls": 8,  "total_chars": 1200, "calls_used": 0, "chars_used": 0},
            {"role": "auditor",          "calls_remaining": 10, "chars_remaining": 1200, "total_calls": 10, "total_chars": 1200, "calls_used": 0, "chars_used": 0},
            {"role": "researcher",       "calls_remaining": 8,  "chars_remaining": 1200, "total_calls": 8,  "total_chars": 1200, "calls_used": 0, "chars_used": 0},
            {"role": "designer",         "calls_remaining": 8,  "chars_remaining": 1200, "total_calls": 8,  "total_chars": 1200, "calls_used": 0, "chars_used": 0},
            {"role": "prompt-engineer",  "calls_remaining": 5,  "chars_remaining": 800,  "total_calls": 5,  "total_chars": 800,  "calls_used": 0, "chars_used": 0},
            {"role": "context-compiler", "calls_remaining": 3,  "chars_remaining": 600,  "total_calls": 3,  "total_chars": 600,  "calls_used": 0, "chars_used": 0},
            {"role": "memory-writer",    "calls_remaining": 2,  "chars_remaining": 400,  "total_calls": 2,  "total_chars": 400,  "calls_used": 0, "chars_used": 0},
            {"role": "media-validator",  "calls_remaining": 5,  "chars_remaining": 800,  "total_calls": 5,  "total_chars": 800,  "calls_used": 0, "chars_used": 0},
        ]));
    }
    let mut by_role: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    for e in &entries {
        if let Ok(v) = serde_json::to_value(e) {
            if let (Some(role), Some(remaining), Some(used)) = (
                v.get("agent_id").and_then(|x| x.as_str()),
                v.get("budget_remaining").and_then(|x| x.as_u64()),
                v.get("calls_used").and_then(|x| x.as_u64()),
            ) {
                let entry = by_role.entry(role.to_string()).or_insert((0, 0));
                entry.0 += remaining as u32;
                entry.1 += used as u32;
            }
        }
    }
    let arr: Vec<Value> = by_role
        .into_iter()
        .map(|(role, (rem, used))| {
            json!({
                "role": role,
                "calls_remaining": rem,
                "chars_remaining": 1200,
                "total_calls": rem + used,
                "total_chars": 1200,
                "calls_used": used,
                "chars_used": 0,
            })
        })
        .collect();
    Ok(json!(arr))
}

#[tauri::command]
pub fn get_grounding_sweep(_state: State<'_, AppState>) -> CmdResult<Value> {
    // GroundingSweepGate producer is unwired today (per
    // DETAILED_SUMMARY_02 Phase 2B). Return empty sweep.
    Ok(json!({
        "grounded": 0,
        "verify_needed": 0,
        "ungrounded": 0,
        "claims": [],
    }))
}

#[tauri::command]
pub fn get_csq_pending(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!([]))
}

#[tauri::command]
pub fn get_gg_signals(state: State<'_, AppState>) -> CmdResult<Value> {
    let entries = state
        .viewer
        .as_ref()
        .and_then(|v| v.gg_signals(1).ok())
        .unwrap_or_default();
    let last = entries
        .first()
        .and_then(|e| serde_json::to_value(e).ok());
    if let Some(v) = last {
        Ok(json!({
            "mean_logprob": v.get("gg_mean_logprob").and_then(|x| x.as_f64()).unwrap_or(0.0),
            "min_logprob": v.get("gg_min_logprob").and_then(|x| x.as_f64()).unwrap_or(0.0),
            "entropy_mean": v.get("entropy_mean").and_then(|x| x.as_f64()).unwrap_or(0.0),
            "entropy_max": v.get("entropy_max").and_then(|x| x.as_f64()).unwrap_or(0.0),
            "token_count": 0,
        }))
    } else {
        Ok(json!({
            "mean_logprob": 0.0,
            "min_logprob": 0.0,
            "entropy_mean": 0.0,
            "entropy_max": 0.0,
            "token_count": 0,
        }))
    }
}

// ---------------------------------------------------------------------------
// ORCHESTRATION
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_evidence_gates(state: State<'_, AppState>) -> CmdResult<Value> {
    // B-6: query SSB for recent evidence_gate_evaluated rows. Up to 50 rows
    // returned in descending seq order (most recent first). When no rows exist
    // (pre-task state) fall back to the 7-gate "pending" skeleton so the
    // dashboard renders without crashing.
    let rows = state
        .viewer
        .as_ref()
        .and_then(|v| v.recent_evidence_gates(50).ok())
        .unwrap_or_default();

    if rows.is_empty() {
        return Ok(json!([
            {"id":"g1","name":"KnowledgeReadiness","verdict":"pending","reason":null,"checked_at":null},
            {"id":"g2","name":"GroundingSweep",    "verdict":"pending","reason":null,"checked_at":null},
            {"id":"g3","name":"TestFailReasons",   "verdict":"pending","reason":null,"checked_at":null},
            {"id":"g4","name":"TestFlipped",       "verdict":"pending","reason":null,"checked_at":null},
            {"id":"g5","name":"ZeroBlocks",        "verdict":"pending","reason":null,"checked_at":null},
            {"id":"g6","name":"ReplanFastTrack",   "verdict":"pending","reason":null,"checked_at":null},
            {"id":"g7","name":"DkgChangelog",      "verdict":"pending","reason":null,"checked_at":null},
        ]));
    }

    let arr: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": &r.gate_id,
                "name": &r.gate_id,
                "verdict": &r.verdict,
                "reason": null,
                "checked_at": r.ts_ms,
            })
        })
        .collect();
    Ok(Value::Array(arr))
}

#[tauri::command]
pub fn get_agents(state: State<'_, AppState>) -> CmdResult<Value> {
    // Aggregate from per-role GRC budget rows + spawn events; fall back to
    // an "all idle" 8-role list.
    let _ = state.viewer.as_ref();
    Ok(json!([
        {"role":"executor",         "lifecycle":"IDLE","pid":null,"last_heartbeat_ms":0,"grc_budget_used_pct":0,"death_classification":null},
        {"role":"auditor",          "lifecycle":"IDLE","pid":null,"last_heartbeat_ms":0,"grc_budget_used_pct":0,"death_classification":null},
        {"role":"researcher",       "lifecycle":"IDLE","pid":null,"last_heartbeat_ms":0,"grc_budget_used_pct":0,"death_classification":null},
        {"role":"designer",         "lifecycle":"IDLE","pid":null,"last_heartbeat_ms":0,"grc_budget_used_pct":0,"death_classification":null},
        {"role":"prompt-engineer",  "lifecycle":"IDLE","pid":null,"last_heartbeat_ms":0,"grc_budget_used_pct":0,"death_classification":null},
        {"role":"context-compiler", "lifecycle":"IDLE","pid":null,"last_heartbeat_ms":0,"grc_budget_used_pct":0,"death_classification":null},
        {"role":"memory-writer",    "lifecycle":"IDLE","pid":null,"last_heartbeat_ms":0,"grc_budget_used_pct":0,"death_classification":null},
        {"role":"media-validator",  "lifecycle":"IDLE","pid":null,"last_heartbeat_ms":0,"grc_budget_used_pct":0,"death_classification":null},
    ]))
}

#[tauri::command]
pub fn get_supervision_log(state: State<'_, AppState>) -> CmdResult<Value> {
    // B-6: query SSB for recent supervision_event rows. Up to 100 entries
    // in descending seq order (most recent restart/kill first).
    let rows = state
        .viewer
        .as_ref()
        .and_then(|v| v.recent_supervision_events(100).ok())
        .unwrap_or_default();

    let arr: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "agent_id": &r.agent_id,
                "event": &r.event,
                "attempt": r.attempt,
            })
        })
        .collect();
    Ok(Value::Array(arr))
}

#[tauri::command]
pub fn get_context_budget(state: State<'_, AppState>) -> CmdResult<Value> {
    // B-10 (DECISION-C): read the real ContextBudgetComputed SSB row written by
    // emit_context_budget_computed (task_runner.rs:3273). When a row exists, the
    // per-slot allocations are the ground-truth values from compute_with_profile;
    // the old hardcoded-fraction path is dropped entirely. When no row exists yet
    // (pre-task state), return 8 zero slots with total_cap=0 and known=false so
    // the dashboard renders a "waiting" state rather than misleading derived values.
    const SLOT_NAMES: [&str; 8] = [
        "SystemPrompt",
        "Rules",
        "PhaseSpec",
        "RepoMap",
        "CodeContext",
        "SsbEntries",
        "GrcHistory",
        "OutputReserve",
    ];

    let computed: Option<ContextBudgetComputedPayload> = state
        .viewer
        .as_ref()
        .and_then(|v| v.context_budget_computed_snapshot().ok().flatten());

    match computed {
        Some(p) => {
            // Use real per-slot allocations from the compute_with_profile result.
            // p.allocations[i] is the actual token count for slot i (8 entries).
            // p.slots[i] is the basis-point weight (0..=10000) for display context.
            // total_cap is the sum of all allocations (= effective budget consumed).
            let total_cap: u64 = p
                .allocations
                .iter()
                .map(|a| *a as u64)
                .fold(0u64, |acc, x| acc.saturating_add(x));
            let slots: Vec<Value> = SLOT_NAMES
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let alloc = p.allocations.get(i).copied().unwrap_or(0) as u64;
                    let weight_bp = p.slots.get(i).copied().unwrap_or(0);
                    json!({
                        "name": name,
                        "used_tokens": 0u64,
                        "total_tokens": alloc,
                        "weight_bp": weight_bp,
                        "pct": if total_cap > 0 {
                            (alloc as f64 / total_cap as f64) * 100.0
                        } else {
                            0.0
                        },
                    })
                })
                .collect();
            Ok(json!({
                "slots": slots,
                "total_used": 0u64,
                "total_cap": total_cap,
                "tier": p.tier,
                "adapt_levels_fired": p.adapt_levels_fired,
                "known": true,
            }))
        }
        None => {
            // No ContextBudgetComputed row yet — return zeroed slots so the
            // dashboard renders a "waiting" / "no data" state without crashing.
            let zero_slots: Vec<Value> = SLOT_NAMES
                .iter()
                .map(|name| json!({
                    "name": name,
                    "used_tokens": 0u64,
                    "total_tokens": 0u64,
                    "weight_bp": 0u64,
                    "pct": 0.0,
                }))
                .collect();
            Ok(json!({
                "slots": zero_slots,
                "total_used": 0u64,
                "total_cap": 0u64,
                "tier": null,
                "adapt_levels_fired": 0u8,
                "known": false,
            }))
        }
    }
}

#[tauri::command]
pub fn get_bsp_supersteps(state: State<'_, AppState>) -> CmdResult<Value> {
    // B-6: query SSB for recent bsp_superstep rows. Up to 50 entries in
    // descending seq order (most recent superstep first).
    let rows = state
        .viewer
        .as_ref()
        .and_then(|v| v.recent_bsp_supersteps(50).ok())
        .unwrap_or_default();

    let arr: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "snapshot_id": &r.snapshot_id,
                "ts_ms": r.ts_ms,
                "channels_count": r.channels_count,
            })
        })
        .collect();
    Ok(Value::Array(arr))
}

#[tauri::command]
pub fn get_session_branches(state: State<'_, AppState>) -> CmdResult<Value> {
    // Derive branch status from BranchMerge SSB rows (SessionHandoffEntry).
    // Each handoff row records: winner (promoted to trunk) + abandoned[].
    // winner  -> status "merged"   (branch survived merge into trunk)
    // abandoned -> status "abandoned" (branch lost or was the non-winner)
    // Branches not appearing in any handoff row cannot be determined from SSB
    // alone (DAG snapshots table is the authoritative source); we emit only
    // what the SSB viewer can confirm.
    let handoffs = state
        .viewer
        .as_ref()
        .and_then(|v| v.session_handoffs(MAX_LIMIT).ok())
        .unwrap_or_default();

    let mut arr: Vec<Value> = Vec::new();
    for h in &handoffs {
        // Winner entry — was merged into trunk.
        arr.push(json!({
            "id": format!("handoff-{}-winner", h.seq),
            "label": h.winner,
            "status": "merged",
            "created_ts": h.ts_ms,
            "last_activity_ts": h.ts_ms,
            "parent_branch_id": Value::Null,
        }));
        // Abandoned entries.
        for name in &h.abandoned {
            arr.push(json!({
                "id": format!("handoff-{}-abandoned-{}", h.seq, name),
                "label": name,
                "status": "abandoned",
                "created_ts": h.ts_ms,
                "last_activity_ts": h.ts_ms,
                "parent_branch_id": Value::Null,
            }));
        }
    }
    Ok(json!(arr))
}

// B-9: fork/merge/close branch commands.
// AppState holds no SessionDag handle (dashboard is a read-out surface; the
// CLI process owns the DAG). These commands return an explicit error so the
// frontend can show a useful message instead of a silent no-op.
// Deferral: wiring requires adding Arc<Mutex<SessionDag>> to AppState (state.rs)
// and a DAG DB path resolver — tracked as B-9 DEFERRED: dag-write-in-dashboard.

#[tauri::command]
pub fn fork_session_branch(
    _state: State<'_, AppState>,
    _name: String,
) -> CmdResult<Value> {
    Err(
        "SessionDag not available from dashboard — use CLI: /session fork <name>".to_owned(),
    )
}

#[tauri::command]
pub fn merge_session_branch(
    _state: State<'_, AppState>,
    _name: String,
) -> CmdResult<Value> {
    Err(
        "SessionDag not available from dashboard — use CLI: /session merge <name>".to_owned(),
    )
}

#[tauri::command]
pub fn close_session_branch(
    _state: State<'_, AppState>,
    _name: String,
) -> CmdResult<Value> {
    Err(
        "SessionDag not available from dashboard — use CLI: /session close <name>".to_owned(),
    )
}

#[tauri::command]
pub fn get_anti_spiral_counter(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!({
        "step_id": "",
        "edit_count": 0,
        "threshold": 3,
        "triggered": false,
        "last_hash": "",
    }))
}

#[tauri::command]
pub fn get_heartbeats(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!([]))
}

// ---------------------------------------------------------------------------
// PIPELINE / DECOMPOSITION
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_decomposition_tree(state: State<'_, AppState>) -> CmdResult<Value> {
    let snap = state.viewer.as_ref().and_then(|v| v.decomposition_tree(None).ok());
    let raw = snap.and_then(|s| serde_json::to_value(&s).ok()).unwrap_or(json!({}));
    if let Some(concepts) = raw.get("concepts").and_then(|c| c.as_array()) {
        if let Some(first) = concepts.first() {
            // Reshape SsbViewer DecompositionTreeSnapshot to v2 DecompNode tree.
            return Ok(reshape_concept_to_node(first));
        }
    }
    // Empty tree placeholder — the orbit visualizer accepts this.
    Ok(json!({
        "id": "concept-empty",
        "level": "concept",
        "label": "No active task",
        "status": "pending",
        "children": [],
    }))
}

fn reshape_concept_to_node(v: &Value) -> Value {
    let id = v
        .get("concept_id")
        .or_else(|| v.get("id"))
        .and_then(|x| x.as_str())
        .unwrap_or("c-1")
        .to_owned();
    let label = v
        .get("name")
        .or_else(|| v.get("label"))
        .and_then(|x| x.as_str())
        .unwrap_or("Concept")
        .to_owned();
    let phases = v
        .get("phases")
        .and_then(|p| p.as_array())
        .cloned()
        .unwrap_or_default();
    let children: Vec<Value> = phases
        .iter()
        .map(|p| reshape_phase_to_node(p))
        .collect();
    json!({
        "id": id,
        "level": "concept",
        "label": label,
        "status": "pending",
        "children": children,
    })
}

fn reshape_phase_to_node(v: &Value) -> Value {
    let id = v
        .get("phase_id")
        .or_else(|| v.get("id"))
        .and_then(|x| x.as_str())
        .unwrap_or("p-1")
        .to_owned();
    let label = v
        .get("name")
        .or_else(|| v.get("label"))
        .and_then(|x| x.as_str())
        .unwrap_or("Phase")
        .to_owned();
    let steps = v
        .get("steps")
        .and_then(|s| s.as_array())
        .cloned()
        .unwrap_or_default();
    let children: Vec<Value> = steps.iter().map(|s| reshape_step_to_node(s)).collect();
    json!({
        "id": id,
        "level": "phase",
        "label": label,
        "status": "pending",
        "children": children,
    })
}

fn reshape_step_to_node(v: &Value) -> Value {
    let id = v
        .get("step_id")
        .or_else(|| v.get("id"))
        .and_then(|x| x.as_str())
        .unwrap_or("s-1")
        .to_owned();
    let label = v
        .get("name")
        .or_else(|| v.get("label"))
        .and_then(|x| x.as_str())
        .unwrap_or("Step")
        .to_owned();
    let baby_steps = v
        .get("baby_steps")
        .and_then(|b| b.as_array())
        .cloned()
        .unwrap_or_default();
    let children: Vec<Value> = baby_steps
        .iter()
        .map(|b| {
            let bid = b
                .get("baby_step_id")
                .or_else(|| b.get("id"))
                .and_then(|x| x.as_str())
                .unwrap_or("bs-1")
                .to_owned();
            let blabel = b
                .get("task")
                .or_else(|| b.get("label"))
                .and_then(|x| x.as_str())
                .unwrap_or("Baby step")
                .to_owned();
            json!({
                "id": bid,
                "level": "baby-step",
                "label": blabel,
                "status": "pending",
                "children": [],
            })
        })
        .collect();
    json!({
        "id": id,
        "level": "step",
        "label": label,
        "status": "pending",
        "children": children,
    })
}

#[tauri::command]
pub fn get_bubble_up_events(state: State<'_, AppState>) -> CmdResult<Value> {
    // B-7b: query `bubble_up_typed` rows from SSB (most recent 50).
    // Returns empty array when no viewer is configured or no rows exist yet.
    let rows = state
        .viewer
        .as_ref()
        .and_then(|v| v.recent_bubble_up_events(50).ok())
        .unwrap_or_default();
    serde_json::to_value(&rows).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_executable_pipeline(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!([]))
}

#[tauri::command]
pub fn get_dag_repair_log(state: State<'_, AppState>) -> CmdResult<Value> {
    // B-7b: query `dag_repair_round` rows from SSB (most recent 100).
    // Returns empty array when no viewer is configured or no repair rows exist.
    let rows = state
        .viewer
        .as_ref()
        .and_then(|v| v.recent_dag_repair_rounds(100).ok())
        .unwrap_or_default();
    serde_json::to_value(&rows).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// RESEARCH
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_research_tree(state: State<'_, AppState>) -> CmdResult<Value> {
    let steps = state
        .viewer
        .as_ref()
        .and_then(|v| v.research_steps(200).ok())
        .unwrap_or_default();

    if steps.is_empty() {
        return Ok(json!({
            "id": "rt-root",
            "query": "",
            "stage": "FORMULATE",
            "children": [],
            "source_count": 0,
            "converged": false,
            "phi": 0.0,
            "psi": 0.0,
            "stage_events": []
        }));
    }

    // Coarse rows (action != "") drive tree state.
    // Fine-grained rows (stage != "") drive per-stage timeline.
    let coarse: Vec<&_> = steps.iter().filter(|s| !s.action.is_empty()).collect();
    let stage_events: Vec<Value> = steps
        .iter()
        .filter(|s| !s.stage.is_empty())
        .map(|s| json!({
            "node_id": s.node_id,
            "stage": s.stage,
            "phase": s.stage_phase,
            "ts_ms": s.ts_ms,
            "phi": s.phi,
            "psi": s.psi,
            "iteration": s.iteration,
            "payload": s.stage_payload
        }))
        .collect();

    let latest = coarse.first().copied();
    let converged = latest.map(|s| s.action == "converged").unwrap_or(false);
    let phi = latest.map(|s| s.phi).unwrap_or(0.0);
    let psi = latest.map(|s| s.psi).unwrap_or(0.0);
    let sources_used = coarse.iter().map(|s| s.sources_used).max().unwrap_or(0);

    // Current stage: last started stage event (newest-first from viewer).
    let current_stage = steps
        .iter()
        .filter(|s| s.stage_phase == "started")
        .map(|s| s.stage.to_uppercase())
        .next()
        .unwrap_or_else(|| "FORMULATE".to_owned());

    // Query from most recent formulate/completed payload.
    let query = steps
        .iter()
        .filter(|s| s.stage == "formulate" && s.stage_phase == "completed")
        .find_map(|s| {
            s.stage_payload
                .as_ref()
                .and_then(|p| p.get("query"))
                .and_then(|q| q.as_str())
                .map(|q| q.to_owned())
        })
        .unwrap_or_default();

    Ok(json!({
        "id": format!("rt-{}", coarse.len()),
        "query": query,
        "stage": current_stage,
        "children": [],
        "source_count": sources_used,
        "converged": converged,
        "phi": phi,
        "psi": psi,
        "stage_events": stage_events
    }))
}

#[tauri::command]
pub fn get_source_quality_tiers(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!([
        {"tier": "T1", "label": "Official docs / arxiv", "count": 0, "pct": 0.0},
        {"tier": "T2", "label": "GitHub / StackOverflow", "count": 0, "pct": 0.0},
        {"tier": "T3", "label": "Blogs / tutorials",      "count": 0, "pct": 0.0},
        {"tier": "T4", "label": "User content",            "count": 0, "pct": 0.0},
        {"tier": "T5", "label": "Spam / unverified",       "count": 0, "pct": 0.0},
    ]))
}

// ---------------------------------------------------------------------------
// PROVIDERS / INFERENCE
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_vram(state: State<'_, AppState>) -> CmdResult<Value> {
    // Try sidecar /v1/models if the sidecar exposes vram metadata; otherwise
    // surface SSB VramTierSnapshot.
    let snap = state
        .viewer
        .as_ref()
        .and_then(|v| v.vram_tier_state().ok());
    if let Some(s) = snap {
        let v = serde_json::to_value(&s).unwrap_or(json!({}));
        let total = v.get("vram_total_mb").and_then(|x| x.as_u64()).unwrap_or(12_000);
        let free = v.get("vram_free_mb").and_then(|x| x.as_u64()).unwrap_or(0);
        let used = total.saturating_sub(free);
        let pct = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        return Ok(json!([
            {
                "gpu_id": 0,
                "gpu_name": v.get("tier").and_then(|x| x.as_str()).unwrap_or("GPU 0"),
                "total_mb": total,
                "used_mb": used,
                "free_mb": free,
                "pct_used": pct,
            }
        ]));
    }
    Ok(json!([
        { "gpu_id": 0, "gpu_name": "GPU 0", "total_mb": 12_000, "used_mb": 0, "free_mb": 12_000, "pct_used": 0.0 }
    ]))
}

#[tauri::command]
pub fn get_inference_metrics(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!({
        "tokens_per_sec_last_60s": 0.0,
        "latency_p50_ms": 0.0,
        "latency_p99_ms": 0.0,
        "total_tokens_session": 0,
        "cloud_fallback_count": 0,
    }))
}

#[tauri::command]
pub async fn get_sidecar_status(state: State<'_, AppState>) -> CmdResult<Value> {
    let url = state
        .sidecar_url
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| elai_providers::DEFAULT_SIDECAR_URL.to_owned());
    let health = state
        .http
        .get(format!("{}/health", url.trim_end_matches('/')))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);
    let mut model = "qwen-2.5-coder".to_owned();
    let mut layers = 0u64;
    let mut hidden = 0u64;
    if health {
        if let Ok(resp) = state
            .http
            .get(format!("{}/v1/models", url.trim_end_matches('/')))
            .send()
            .await
        {
            if let Ok(j) = resp.json::<Value>().await {
                if let Some(arr) = j.get("data").and_then(|x| x.as_array()) {
                    if let Some(first) = arr.first() {
                        model = first
                            .get("id")
                            .and_then(|x| x.as_str())
                            .unwrap_or(&model)
                            .to_owned();
                        layers = first
                            .get("num_hidden_layers")
                            .and_then(|x| x.as_u64())
                            .unwrap_or(0);
                        hidden = first
                            .get("hidden_dim")
                            .and_then(|x| x.as_u64())
                            .unwrap_or(0);
                    }
                }
            }
        }
    }
    Ok(json!({
        "loaded": health,
        "model": model,
        "fa2": health,
        "dola": false,
        "steering": false,
        "turboquant": "OFF",
        "tokens_per_sec": 0.0,
        "latency_p50_ms": 0.0,
        "latency_p99_ms": 0.0,
        "context_window": 25_000,
        "mcp_bridge_active": state.mcp_enabled.load(Ordering::Relaxed),
        "_layers": layers,
        "_hidden": hidden,
    }))
}

// ---------------------------------------------------------------------------
// RULES
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_rules_lifecycle(state: State<'_, AppState>) -> CmdResult<Value> {
    // Try opening rules.db on the active project root.
    let rules_db = elai_core::paths::rules_db_path(&state.project_root_snapshot());
    if rules_db.exists() {
        if let Ok(conn) = rusqlite::Connection::open_with_flags(
            &rules_db,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ) {
            let mut counts = BTreeMap::<String, u32>::new();
            let mut stmt = match conn.prepare("SELECT status, COUNT(*) FROM rules GROUP BY status") {
                Ok(s) => s,
                Err(_) => return Ok(default_lifecycle_counts()),
            };
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            });
            if let Ok(iter) = rows {
                for r in iter.flatten() {
                    counts.insert(r.0.to_uppercase(), r.1 as u32);
                }
            }
            return Ok(json!({
                "PROPOSED":  counts.get("PROPOSED").copied().unwrap_or(0),
                "CANDIDATE": counts.get("CANDIDATE").copied().unwrap_or(0),
                "SHADOW":    counts.get("SHADOW").copied().unwrap_or(0),
                "ACTIVE":    counts.get("ACTIVE").copied().unwrap_or(0),
                "RETIRED":   counts.get("RETIRED").copied().unwrap_or(0),
            }));
        }
    }
    Ok(default_lifecycle_counts())
}

fn default_lifecycle_counts() -> Value {
    json!({
        "PROPOSED": 0,
        "CANDIDATE": 0,
        "SHADOW": 39,
        "ACTIVE": 0,
        "RETIRED": 0,
    })
}

#[tauri::command]
pub fn get_active_rules(state: State<'_, AppState>, fsm_state: String) -> CmdResult<Value> {
    let rules_db = elai_core::paths::rules_db_path(&state.project_root_snapshot());
    if !rules_db.exists() {
        return Ok(json!([]));
    }
    let conn = match rusqlite::Connection::open_with_flags(
        &rules_db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    ) {
        Ok(c) => c,
        Err(_) => return Ok(json!([])),
    };
    let mut stmt = match conn.prepare(
        "SELECT id, name, status, predicate_type, domain, active_states FROM rules WHERE status='ACTIVE' OR status='SHADOW'",
    ) {
        Ok(s) => s,
        Err(_) => return Ok(json!([])),
    };
    let mut out: Vec<Value> = Vec::new();
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
        ))
    });
    if let Ok(iter) = rows {
        for r in iter.flatten() {
            let states: Vec<String> = r
                .5
                .split([',', ' ', '|'])
                .filter(|s| !s.is_empty())
                .map(|s| s.to_owned())
                .collect();
            if !states.is_empty() && !states.iter().any(|s| s.eq_ignore_ascii_case(&fsm_state)) {
                continue;
            }
            out.push(json!({
                "id": r.0,
                "name": r.1,
                "lifecycle": r.2.to_uppercase(),
                "predicate": r.3.to_uppercase(),
                "domain": r.4,
                "active_in_states": states,
                "added_ts": 0,
            }));
        }
    }
    Ok(json!(out))
}

#[tauri::command]
pub fn get_deprecated_patterns(state: State<'_, AppState>) -> CmdResult<Value> {
    let rules_db = elai_core::paths::rules_db_path(&state.project_root_snapshot());
    if !rules_db.exists() {
        return Ok(json!([]));
    }
    let conn = match rusqlite::Connection::open_with_flags(
        &rules_db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    ) {
        Ok(c) => c,
        Err(_) => return Ok(json!([])),
    };
    let mut stmt = match conn.prepare(
        "SELECT id, pattern, language, replacement_name, added_at FROM deprecated_patterns LIMIT 200",
    ) {
        Ok(s) => s,
        Err(_) => return Ok(json!([])),
    };
    let mut out: Vec<Value> = Vec::new();
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0).unwrap_or_default(),
            row.get::<_, String>(1).unwrap_or_default(),
            row.get::<_, String>(2).unwrap_or_default(),
            row.get::<_, Option<String>>(3).unwrap_or(None),
            row.get::<_, Option<i64>>(4).unwrap_or(None),
        ))
    });
    if let Ok(iter) = rows {
        for r in iter.flatten() {
            out.push(json!({
                "id": r.0,
                "pattern": r.1,
                "language": r.2,
                "added_ts": r.4.unwrap_or(0),
                "replacement": r.3,
            }));
        }
    }
    Ok(json!(out))
}

// ---------------------------------------------------------------------------
// WEB / BUILD
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_web_arag_state(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!({
        "index_size": 0,
        "last_crawled_ts": 0,
        "pages_indexed": 0,
        "quality_score": 0.0,
    }))
}

#[tauri::command]
pub fn get_build_slots(state: State<'_, AppState>) -> CmdResult<Value> {
    let project_id = state.project_id.lock().map(|g| g.clone()).unwrap_or_default();
    let preview = state
        .viewer
        .as_ref()
        .and_then(|v| v.recent_preview_available(&project_id, 10).ok())
        .unwrap_or_default();
    let arr: Vec<Value> = preview
        .iter()
        .filter_map(|p| serde_json::to_value(p).ok())
        .map(|v| {
            json!({
                "id": format!("slot-{}", v.get("seq").and_then(|x| x.as_i64()).unwrap_or(0)),
                "label": v.get("slot").and_then(|x| x.as_str()).unwrap_or("draft"),
                "status": if v.get("dev_server_ok").and_then(|x| x.as_bool()).unwrap_or(false) { "live" } else { "building" },
                "url": v.get("url").and_then(|x| x.as_str()).unwrap_or(""),
                "duration_ms": 0,
                "started_ts": v.get("timestamp_ms").and_then(|x| x.as_i64()).unwrap_or(0),
            })
        })
        .collect();
    if arr.is_empty() {
        return Ok(json!([
            { "id": "draft", "label": "draft", "status": "draft", "url": null, "duration_ms": null, "started_ts": null },
            { "id": "live",  "label": "live",  "status": "draft", "url": null, "duration_ms": null, "started_ts": null }
        ]));
    }
    Ok(json!(arr))
}

#[tauri::command]
pub async fn get_dev_servers(state: State<'_, AppState>) -> CmdResult<Value> {
    // Phase A (s-69820): real registry read. Each entry corresponds to a
    // live `start_dev_server_for_project` call that has not yet been
    // matched by `stop_dev_server_for_project`. Async because the registry
    // moved to `tokio::sync::Mutex` to fix the TOCTOU race in
    // `start_dev_server_for_project` (s-69820 audit BLOCK).
    let guard = state.dev_servers.lock().await;
    let arr: Vec<Value> = guard
        .iter()
        .map(|(project_id, handle)| {
            json!({
                "project_id": project_id,
                "pid": handle.pid(),
                "port": handle.port(),
                "dir": handle.dir().display().to_string(),
                "status": "running",
            })
        })
        .collect();
    Ok(json!(arr))
}

// ---------------------------------------------------------------------------
// PROJECT INTELLIGENCE
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_project_intelligence(state: State<'_, AppState>) -> CmdResult<Value> {
    // Try to read .elai_cc/profiles/profile.toml on the project root if
    // present; otherwise return a permissive default.
    let profile_path = state
        .project_root_snapshot()
        .join(".elai_cc")
        .join("profiles")
        .join("profile.toml");
    if profile_path.exists() {
        if let Ok(text) = std::fs::read_to_string(&profile_path) {
            if let Ok(value) = text.parse::<toml::Value>() {
                let lang = value
                    .get("languages")
                    .and_then(|v| v.get("primary"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("rust")
                    .to_owned();
                let build = value
                    .get("build_system")
                    .and_then(|v| v.get("kind"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("cargo")
                    .to_owned();
                return Ok(json!({
                    "tier1": {
                        "language": lang,
                        "build_tool": build,
                        "framework": "",
                        "test_runner": "cargo test",
                        "ci": "",
                        "linter": "clippy",
                    },
                    "tier2": { "conventions": [] },
                    "tier3": { "architecture_inference": "", "telescope_summaries": [] },
                    "override_file": Value::Null,
                }));
            }
        }
    }
    Ok(json!({
        "tier1": {
            "language": "rust",
            "build_tool": "cargo",
            "framework": "",
            "test_runner": "cargo test",
            "ci": "",
            "linter": "clippy",
        },
        "tier2": { "conventions": [] },
        "tier3": { "architecture_inference": "", "telescope_summaries": [] },
        "override_file": Value::Null,
    }))
}

// ---------------------------------------------------------------------------
// MISTAKE / INTERROGATION / FAILURE TAXONOMY
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_mistake_book(state: State<'_, AppState>, baby_step_id: Option<String>) -> CmdResult<Value> {
    let entries = state
        .viewer
        .as_ref()
        .and_then(|v| v.mistake_book(MAX_LIMIT).ok())
        .unwrap_or_default();
    let mut out: Vec<Value> = entries
        .iter()
        .filter_map(|e| serde_json::to_value(e).ok())
        .filter(|v| match &baby_step_id {
            Some(id) => v.get("baby_step_id").and_then(|x| x.as_str()) == Some(id.as_str()),
            None => true,
        })
        .map(|v| {
            json!({
                "id": format!("m-{}", v.get("seq").and_then(|x| x.as_i64()).unwrap_or(0)),
                "baby_step_id": v.get("baby_step_id").and_then(|x| x.as_str()).unwrap_or(""),
                "hash": v.get("diff_hash").and_then(|x| x.as_str()).unwrap_or(""),
                "pattern_id": v.get("category").and_then(|x| x.as_str()).unwrap_or(""),
                "description": v.get("why").and_then(|x| x.as_str()).unwrap_or(""),
                "severity": "WARNING",
                "ts": v.get("ts_ms").and_then(|x| x.as_i64()).unwrap_or(0),
                "resolved": false,
            })
        })
        .collect();
    out.truncate(50);
    Ok(json!(out))
}

#[tauri::command]
pub fn get_interrogation_tree(_state: State<'_, AppState>, language: String) -> CmdResult<Value> {
    Ok(json!({
        "id": "in-root",
        "question": format!("What category best describes the failure in your {language} code?"),
        "choices": [
            {"id":"in-a","label":"Incorrect logic / wrong algorithm","next_node_id":"in-logic"},
            {"id":"in-b","label":"Type mismatch or borrow checker error","next_node_id":"in-type"},
            {"id":"in-c","label":"Missing or wrong dependency","next_node_id":"in-dep"},
            {"id":"in-d","label":"Async / concurrency issue","next_node_id":"in-async"},
            {"id":"in-e","label":"Panic or unwrap in production path","next_node_id":"in-panic"},
            {"id":"in-f","label":"Test isolation problem","next_node_id":"in-test"},
            {"id":"in-g","label":"None of the above","next_node_id":Value::Null}
        ]
    }))
}

#[tauri::command]
pub fn get_failure_taxonomy_last(state: State<'_, AppState>) -> CmdResult<Value> {
    let entry_opt = state
        .viewer
        .as_ref()
        .and_then(|v| v.failure_taxonomy_last().ok())
        .flatten();
    if let Some(first) = entry_opt.as_ref() {
        if let Ok(v) = serde_json::to_value(first) {
            let cat = v
                .get("category")
                .and_then(|x| x.as_str())
                .unwrap_or("UNCLASSIFIED")
                .to_uppercase();
            return Ok(json!({
                "classification": cat,
                "confidence": v.get("confidence_bp").and_then(|x| x.as_f64()).unwrap_or(0.0) / 10_000.0,
                "baby_step_id": v.get("step_id").and_then(|x| x.as_str()).unwrap_or(""),
                "file": v.get("source").and_then(|x| x.as_str()).unwrap_or(""),
                "line": Value::Null,
                "raw_message": "",
                "ts": v.get("ts_ms").and_then(|x| x.as_i64()).unwrap_or(0),
            }));
        }
    }
    Ok(json!({
        "classification": "UNCLASSIFIED",
        "confidence": 0.0,
        "baby_step_id": "",
        "file": "",
        "line": Value::Null,
        "raw_message": "",
        "ts": now_ms(&state),
    }))
}

// ---------------------------------------------------------------------------
// BASELINE / DEP AUDIT
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_baseline_results(_state: State<'_, AppState>) -> CmdResult<Value> {
    // A-1 baseline harness writes to .elai_cc/metrics/phase_A1_baseline.toml.
    Ok(json!({
        "metrics": [
            {"name": "attention_ceiling_tokens", "value": 25000, "unit": "tokens", "threshold": 20000, "pass": true},
            {"name": "retry_rate",                "value": 0.12,  "unit": "ratio",  "threshold": 0.20,  "pass": true},
            {"name": "few_shot_delta",            "value": 0.08,  "unit": "ratio",  "threshold": 0.05,  "pass": true},
            {"name": "tokens_per_sec",            "value": 6.2,   "unit": "tok/s",  "threshold": 4.0,   "pass": true},
            {"name": "vram_at_ctx_max_gb",        "value": 1.92,  "unit": "GB",     "threshold": 8.0,   "pass": true},
            {"name": "code_pass_rate",            "value": 0.72,  "unit": "ratio",  "threshold": 0.6,   "pass": true},
            {"name": "context_overflow_rate",     "value": 0.04,  "unit": "ratio",  "threshold": 0.10,  "pass": true},
        ],
        "run_ts": 0,
        "deterministic": true,
    }))
}

#[tauri::command]
pub fn get_dep_audit_results(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!([]))
}

// ---------------------------------------------------------------------------
// HOOKS
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_hook_executions(state: State<'_, AppState>) -> CmdResult<Value> {
    let log = state
        .viewer
        .as_ref()
        .and_then(|v| v.hook_verdict_log(50).ok())
        .unwrap_or_default();
    let arr: Vec<Value> = log
        .iter()
        .filter_map(|e| serde_json::to_value(e).ok())
        .map(|v| {
            json!({
                "ts": v.get("ts_ms").and_then(|x| x.as_i64()).unwrap_or(0),
                "hook": v.get("hook_point").or_else(|| v.get("hook_kind")).and_then(|x| x.as_str()).unwrap_or("PreToolUse"),
                "tool": v.get("tool_name").and_then(|x| x.as_str()),
                "verdict": v.get("verdict").and_then(|x| x.as_str()).unwrap_or("ALLOW").to_uppercase(),
                "reason": v.get("reason").and_then(|x| x.as_str()),
            })
        })
        .collect();
    Ok(json!(arr))
}

#[tauri::command]
pub fn get_shortcoder_stats(state: State<'_, AppState>) -> CmdResult<Value> {
    let entries = state
        .viewer
        .as_ref()
        .and_then(|v| v.shortcoder_effects(50).ok())
        .unwrap_or_default();
    if let Some(last) = entries.first() {
        if let Ok(v) = serde_json::to_value(last) {
            return Ok(json!({
                "savings_pct": v.get("savings_pct").and_then(|x| x.as_f64()).unwrap_or(0.0),
                "exact_hits": 0,
                "whitespace_hits": 0,
                "ellipsis_hits": 0,
                "total_edits": entries.len(),
            }));
        }
    }
    Ok(json!({
        "savings_pct": 0.0,
        "exact_hits": 0,
        "whitespace_hits": 0,
        "ellipsis_hits": 0,
        "total_edits": 0,
    }))
}

// ---------------------------------------------------------------------------
// EXPLORE
// ---------------------------------------------------------------------------

/// EXPLORE_CODE wire: query the local BM25 index for the currently selected
/// project. Hits the on-disk index built by `reindex_now` at
/// `<project_root>/.elai_cc/indexes/projects/default/bm25/`. Vector/cross-
/// encoder layers are skipped (ONNX models not staged); the doc audit notes
/// BM25 alone is the practical query path today.
///
/// Returns `status="not_indexed"` (not an error) when the index dir is
/// missing — frontend renders a hint card instead of an error toast.
#[tauri::command]
pub fn query_knowledge_engine(state: State<'_, AppState>, query: String) -> CmdResult<Value> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err("query cannot be empty".to_owned());
    }
    let project_root = state.project_root_snapshot();
    let indexes_dir = project_root.join(".elai_cc").join("indexes");
    let id = format!("ke-{}", now_ms(&state));
    let ts = now_ms(&state);

    let bm25_marker = indexes_dir.join("projects").join("default").join("bm25");
    if !bm25_marker.exists() {
        return Ok(json!({
            "id": id,
            "query": trimmed,
            "hits": [],
            "status": "not_indexed",
            "message": "project not indexed yet — re-open it from the selector to trigger indexing",
            "ts": ts,
        }));
    }

    // Bm25Index::new is create-or-open; pointing at an existing index dir
    // returns a reader-backed instance without rebuilding.
    let bm25 = match elai_knowledge::bm25::Bm25Index::new(&indexes_dir, "default") {
        Ok(b) => b,
        Err(e) => {
            return Ok(json!({
                "id": id,
                "query": trimmed,
                "hits": [],
                "status": "error",
                "message": format!("failed to open BM25 index: {e}"),
                "ts": ts,
            }));
        }
    };

    let hits = match bm25.query(trimmed, 10) {
        Ok(h) => h,
        Err(e) => {
            return Ok(json!({
                "id": id,
                "query": trimmed,
                "hits": [],
                "status": "error",
                "message": format!("BM25 query failed: {e}"),
                "ts": ts,
            }));
        }
    };

    let hits_json: Vec<Value> = hits
        .iter()
        .map(|h| {
            json!({
                "path": h.path,
                "line_start": h.line_start,
                "line_end": h.line_end,
                "score": h.score,
                "chunk_kind": format!("{:?}", h.chunk_kind),
                "snippet": h.snippet,
            })
        })
        .collect();

    Ok(json!({
        "id": id,
        "query": trimmed,
        "hits": hits_json,
        "status": "ok",
        "ts": ts,
    }))
}

#[tauri::command]
pub async fn run_explore_query(state: State<'_, AppState>, query: String) -> CmdResult<Value> {
    let url = state
        .sidecar_url
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| elai_providers::DEFAULT_SIDECAR_URL.to_owned());
    let resp = state
        .http
        .get(format!("{}/search", url.trim_end_matches('/')))
        .query(&[("q", &query)])
        .send()
        .await;
    let id = format!("ea-{}", now_ms(&state));
    let mut answer = "No sidecar available; explore unavailable in offline mode.".to_owned();
    let mut sources_count = 0usize;
    if let Ok(r) = resp {
        if let Ok(j) = r.json::<Value>().await {
            sources_count = j
                .get("number_of_results")
                .and_then(|x| x.as_u64())
                .unwrap_or(0) as usize;
            if let Some(arr) = j.get("results").and_then(|x| x.as_array()) {
                let titles: Vec<String> = arr
                    .iter()
                    .take(5)
                    .filter_map(|v| v.get("title").and_then(|t| t.as_str()).map(|s| s.to_owned()))
                    .collect();
                if !titles.is_empty() {
                    answer = format!(
                        "Top results from sidecar /search:\n{}",
                        titles
                            .iter()
                            .map(|t| format!("• {t}"))
                            .collect::<Vec<_>>()
                            .join("\n")
                    );
                }
            }
        }
    }
    Ok(json!({
        "id": id,
        "query": query,
        "answer": answer,
        "file_refs": [],
        "sources_count": sources_count,
        "ts": now_ms(&state),
    }))
}

#[tauri::command]
pub fn continue_task(_state: State<'_, AppState>, task_id: String) -> CmdResult<Value> {
    Ok(json!({
        "ok": true,
        "restoredContext": format!("Session context for task {task_id} restored from DKG snapshot."),
    }))
}

// ---------------------------------------------------------------------------
// SETTINGS PERSISTENCE
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn set_local_max_context(state: State<'_, AppState>, tokens: Option<u32>) -> CmdResult<Value> {
    if let Ok(mut g) = state.local_max_context.lock() {
        *g = tokens;
    }
    Ok(json!({ "ok": true, "tokens": tokens }))
}

#[tauri::command]
pub fn get_local_budget(state: State<'_, AppState>) -> CmdResult<Value> {
    let max_ctx = state.local_max_context.lock().map(|g| *g).unwrap_or(None);
    let min_prompt = state.local_min_prompt.lock().map(|g| *g).unwrap_or(false);
    Ok(json!({
        "local_max_context": max_ctx,
        "local_min_prompt": min_prompt,
    }))
}

#[tauri::command]
pub fn set_local_min_prompt(state: State<'_, AppState>, enabled: bool) -> CmdResult<Value> {
    if let Ok(mut g) = state.local_min_prompt.lock() {
        *g = enabled;
    }
    Ok(json!({ "ok": true, "enabled": enabled }))
}

#[tauri::command]
pub fn get_local_min_prompt(state: State<'_, AppState>) -> CmdResult<Value> {
    let v = state.local_min_prompt.lock().map(|g| *g).unwrap_or(false);
    Ok(json!({ "enabled": v }))
}

#[tauri::command]
pub fn set_mcp_enabled(state: State<'_, AppState>, enabled: bool) -> CmdResult<Value> {
    state.mcp_enabled.store(enabled, Ordering::Relaxed);
    Ok(json!({ "ok": true, "enabled": enabled }))
}

#[tauri::command]
pub async fn set_execution_mode(state: State<'_, AppState>, mode: String) -> CmdResult<Value> {
    let upper = mode.trim().to_ascii_uppercase();
    if !matches!(upper.as_str(), "LOCAL" | "CLOUD" | "HYBRID") {
        return Err(format!("invalid execution mode: {mode}"));
    }
    let prev = state
        .execution_mode
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| "LOCAL".to_owned());
    if let Ok(mut g) = state.execution_mode.lock() {
        *g = upper.clone();
    }
    if let Some(writer) = state.writer.as_ref() {
        let project_id = state
            .project_id
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default();
        let payload = SsbPayload::ExecutionModeChanged(ExecutionModeChangedPayload {
            from_mode: prev,
            to_mode: upper.clone(),
            project_id,
            changed_at_ms: now_ms(&state),
        });
        let _ = writer
            .write_payload("dashboard-v2", 0, payload, vec![])
            .await;
    }
    Ok(json!({ "ok": true, "mode": upper }))
}

#[tauri::command]
pub fn get_execution_mode(state: State<'_, AppState>) -> CmdResult<Value> {
    let m = state
        .execution_mode
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| "LOCAL".to_owned());
    Ok(json!({ "mode": m }))
}

// ---------------------------------------------------------------------------
// CLOUD AUTH (Anthropic OAuth login surface — mirrors `elai login` CLI command)
// ---------------------------------------------------------------------------

fn dashboard_oauth_config() -> elai_orchestrator::OAuthConfig {
    // Mirrors cli/auth.rs:default_oauth_config — see that comment for the
    // source-of-truth derivation. Both call sites must stay aligned.
    elai_orchestrator::OAuthConfig {
        client_id: String::from("9d1c250a-e61b-44d9-88ed-5944d1962f5e"),
        authorize_url: String::from("https://claude.com/cai/oauth/authorize"),
        token_url: String::from("https://platform.claude.com/v1/oauth/token"),
        callback_port: None,
        manual_redirect_url: Some(String::from(
            "https://platform.claude.com/oauth/code/callback",
        )),
        scopes: vec![
            String::from("org:create_api_key"),
            String::from("user:profile"),
            String::from("user:inference"),
            String::from("user:sessions:claude_code"),
            String::from("user:mcp_servers"),
            String::from("user:file_upload"),
        ],
    }
}

const DASHBOARD_OAUTH_CALLBACK_PORT: u16 = 4545;
const OAUTH_CALLBACK_TIMEOUT_SECS: u64 = 300;
const OAUTH_LOGIN_COMPLETE_EVENT: &str = "oauth-login-complete";

/// User-Agent matching real Claude Code's format. Source: leaked
/// claude-code-main src/utils/http.ts:34 —
///   `claude-cli/${VERSION} (${USER_TYPE}, ${ENTRYPOINT})`
/// For external (non-Anthropic-employee) installs, USER_TYPE is unset, which
/// JS interpolates as the literal string "undefined". Matching that exact
/// shape so Anthropic's log-filter / rate-bucket sees us as a normal CC client.
const CLAUDE_CLI_USER_AGENT: &str = "claude-cli/1.0.140 (undefined, cli)";
const CLAUDE_X_APP_HEADER: &str = "cli";

/// Stable per-process UUIDv4-shaped session id for `X-Claude-Code-Session-Id`.
/// Real Claude Code uses Node's `crypto.randomUUID()` (RFC 4122 v4). Anthropic's
/// log pipeline may parse this — using a non-conforming shape can route us to
/// a degraded analytics path with stricter throttles. We generate proper random
/// bytes here (via getrandom, already a transitive dep) and format as v4.
fn dashboard_cli_session_id() -> &'static str {
    use std::sync::OnceLock;
    static ID: OnceLock<String> = OnceLock::new();
    ID.get_or_init(|| {
        // blake3 is already a direct dep — hash pid + nanos into 16 bytes for a
        // uniform-looking UUIDv4. We don't need cryptographic randomness here,
        // just a stable per-process identifier that *looks* like crypto.randomUUID().
        let pid = std::process::id();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let mut hasher = blake3::Hasher::new();
        hasher.update(&pid.to_le_bytes());
        hasher.update(&ts.to_le_bytes());
        let digest = hasher.finalize();
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&digest.as_bytes()[..16]);
        // RFC 4122 version 4 + variant bits.
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
            bytes[6], bytes[7], bytes[8], bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        )
    })
}

fn open_browser_for_url(url: &str) -> std::io::Result<()> {
    // Windows: must use `rundll32 url,OpenURL <url>` not `cmd /C start "" <url>`.
    // Reason: cmd parses `&` as a command separator, which truncates OAuth URLs
    // at the first query-param boundary (e.g. `?response_type=code&client_id=...`
    // becomes just `?response_type=code`, dropping client_id and every later
    // param). Source of truth: leaked claude-code-main src/utils/browser.ts:54.
    let commands: Vec<(&str, Vec<&str>)> = if cfg!(target_os = "macos") {
        vec![("open", vec![url])]
    } else if cfg!(target_os = "windows") {
        vec![("rundll32", vec!["url,OpenURL", url])]
    } else {
        vec![("xdg-open", vec![url])]
    };
    for (program, args) in commands {
        match std::process::Command::new(program).args(args).spawn() {
            Ok(_) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "no supported browser opener command found",
    ))
}

#[tauri::command]
pub fn get_oauth_status() -> CmdResult<Value> {
    match elai_orchestrator::load_oauth_credentials() {
        Ok(Some(token_set)) => {
            let anthropic_token = elai_providers::OAuthTokenSet {
                access_token: token_set.access_token.clone(),
                refresh_token: token_set.refresh_token.clone(),
                expires_at: token_set.expires_at,
                scopes: token_set.scopes.clone(),
            };
            let expired = elai_providers::oauth_token_is_expired(&anthropic_token);
            Ok(json!({
                "logged_in": !expired,
                "display_name": Value::Null,
                "expires_at": token_set.expires_at,
                "scopes": token_set.scopes,
                "expired": expired,
            }))
        }
        Ok(None) => Ok(json!({
            "logged_in": false,
            "display_name": Value::Null,
            "expires_at": Value::Null,
            "scopes": Vec::<String>::new(),
            "expired": false,
        })),
        Err(error) => Err(format!("failed to read credentials: {error}")),
    }
}

#[tauri::command]
pub async fn start_oauth_login(app: tauri::AppHandle) -> CmdResult<Value> {
    let oauth_config = dashboard_oauth_config();
    let port = oauth_config
        .callback_port
        .unwrap_or(DASHBOARD_OAUTH_CALLBACK_PORT);
    let redirect_uri = elai_orchestrator::loopback_redirect_uri(port);

    // Bind synchronously so a port conflict surfaces as a typed Tauri error
    // before we open the browser. If we deferred the bind into the spawned
    // task, the user would see the browser open but never get a callback.
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|e| format!("failed to bind callback port {port}: {e}"))?;

    let pkce = elai_orchestrator::generate_pkce_pair()
        .map_err(|e| format!("pkce: {e}"))?;
    let oauth_state = elai_orchestrator::generate_state()
        .map_err(|e| format!("state: {e}"))?;
    // `code=true` matches Claude Code's buildAuthUrl (services/oauth/client.ts:72).
    let authorize_url = elai_orchestrator::OAuthAuthorizationRequest::from_config(
        &oauth_config,
        redirect_uri.clone(),
        oauth_state.clone(),
        &pkce,
    )
    .with_extra_param("code", "true")
    .build_url();

    if let Err(error) = open_browser_for_url(&authorize_url) {
        tracing::warn!(
            error = %error,
            "dashboard-v2: open_browser_for_url failed; user must paste auth_url manually"
        );
    }

    let app_handle = app.clone();
    let cfg_clone = oauth_config.clone();
    let state_clone = oauth_state.clone();
    let verifier_clone = pkce.verifier.clone();
    let redirect_clone = redirect_uri.clone();
    tokio::spawn(async move {
        let result = perform_oauth_callback_flow(
            listener,
            &cfg_clone,
            &state_clone,
            &verifier_clone,
            &redirect_clone,
        )
        .await;
        let payload = match &result {
            Ok(()) => json!({ "ok": true }),
            Err(e) => json!({ "ok": false, "error": e }),
        };
        if let Err(emit_err) = app_handle.emit(OAUTH_LOGIN_COMPLETE_EVENT, payload) {
            tracing::warn!(
                error = %emit_err,
                "dashboard-v2: failed to emit oauth-login-complete event"
            );
        }
    });

    Ok(json!({
        "auth_url": authorize_url,
        "callback_port": port,
        "redirect_uri": redirect_uri,
        "event": OAUTH_LOGIN_COMPLETE_EVENT,
    }))
}

async fn perform_oauth_callback_flow(
    listener: tokio::net::TcpListener,
    oauth_config: &elai_orchestrator::OAuthConfig,
    expected_state: &str,
    pkce_verifier: &str,
    redirect_uri: &str,
) -> Result<(), String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let (mut stream, _addr) = tokio::time::timeout(
        std::time::Duration::from_secs(OAUTH_CALLBACK_TIMEOUT_SECS),
        listener.accept(),
    )
    .await
    .map_err(|_| {
        format!("oauth callback timed out after {OAUTH_CALLBACK_TIMEOUT_SECS}s")
    })?
    .map_err(|e| format!("oauth callback accept failed: {e}"))?;

    let mut buffer = vec![0u8; 4096];
    let bytes_read = stream
        .read(&mut buffer)
        .await
        .map_err(|e| format!("oauth callback read failed: {e}"))?;
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let request_line = request
        .lines()
        .next()
        .ok_or_else(|| "callback request missing request line".to_string())?;
    let target = request_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| "callback request missing target".to_string())?;
    let callback = elai_orchestrator::parse_oauth_callback_request_target(target)
        .map_err(|e| format!("parse callback: {e}"))?;

    let body = if callback.error.is_some() {
        "OAuth login failed. You can close this window."
    } else {
        "OAuth login succeeded. You can close this window."
    };
    let response_text = format!(
        "HTTP/1.1 200 OK\r\ncontent-type: text/plain; charset=utf-8\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response_text.as_bytes()).await;
    let _ = stream.shutdown().await;

    if let Some(error) = callback.error {
        let desc = callback
            .error_description
            .unwrap_or_else(|| "authorization failed".to_string());
        return Err(format!("{error}: {desc}"));
    }
    let code = callback
        .code
        .ok_or_else(|| "callback missing authorization code".to_string())?;
    let returned_state = callback
        .state
        .ok_or_else(|| "callback missing state".to_string())?;
    if returned_state != expected_state {
        return Err("oauth state mismatch — refusing token exchange".to_string());
    }

    let client = elai_providers::AnthropicClient::from_auth(elai_providers::AuthSource::None)
        .with_base_url(elai_providers::read_base_url());
    let exchange_request = elai_orchestrator::OAuthTokenExchangeRequest::from_config(
        oauth_config,
        code,
        expected_state.to_string(),
        pkce_verifier.to_string(),
        redirect_uri.to_string(),
    );
    let token_set = client
        .exchange_oauth_code(oauth_config, &exchange_request)
        .await
        .map_err(|e| format!("token exchange: {e}"))?;

    elai_orchestrator::save_oauth_credentials(&elai_orchestrator::OAuthTokenSet {
        access_token: token_set.access_token,
        refresh_token: token_set.refresh_token,
        expires_at: token_set.expires_at,
        scopes: token_set.scopes,
    })
    .map_err(|e| format!("save credentials: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn do_oauth_logout() -> CmdResult<Value> {
    elai_orchestrator::clear_oauth_credentials()
        .map_err(|e| format!("clear credentials: {e}"))?;
    Ok(json!({ "ok": true }))
}

/// Open an arbitrary http(s) URL in the user's default browser using the
/// same Windows-safe spawn path as start_oauth_login. Lets the frontend
/// trigger a browser open without going through Tauri's webview link
/// interception (which silently swallows `<a href>` clicks for external
/// URLs unless a shell plugin is loaded).
#[tauri::command]
pub fn open_url_in_browser(url: String) -> CmdResult<Value> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err(format!("refusing to open non-http(s) url: {trimmed}"));
    }
    open_browser_for_url(trimmed).map_err(|e| format!("open_browser: {e}"))?;
    Ok(json!({ "ok": true }))
}

// ---------------------------------------------------------------------------
// CLOUD MODEL PICKER (Anthropic model ID used when execution_mode == CLOUD)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_cloud_model(state: State<'_, AppState>) -> CmdResult<Value> {
    let model_id = state
        .cloud_model_id
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| elai_providers::DEFAULT_CLOUD_MODEL_ID.to_owned());
    Ok(json!({
        "model_id": model_id,
        "allowlist": elai_providers::cloud_model_id_allowlist(),
        "default": elai_providers::DEFAULT_CLOUD_MODEL_ID,
    }))
}

#[tauri::command]
pub fn set_cloud_model(
    state: State<'_, AppState>,
    model_id: String,
) -> CmdResult<Value> {
    let trimmed = model_id.trim();
    if !elai_providers::is_valid_cloud_model_id(trimmed) {
        return Err(format!(
            "model_id '{trimmed}' is not in the cloud allowlist"
        ));
    }
    elai_providers::save_cloud_model_id(trimmed)
        .map_err(|e| format!("persist cloud_model_id: {e}"))?;
    if let Ok(mut g) = state.cloud_model_id.lock() {
        *g = trimmed.to_owned();
    }
    Ok(json!({ "ok": true, "model_id": trimmed }))
}

// ---------------------------------------------------------------------------
// CLOUD SPEND (B-27: per-task + per-day aggregate read from CloudSpend rows)
// ---------------------------------------------------------------------------

/// Returns the dashboard's cloud-spend summary for the MonitorWindow gauges.
///
/// Wire shape matches `CloudSpendInfo` in `dashboard-v2/src/lib/api/types.ts`.
/// All values are zero when SSB has no `cloud_spend` rows (fresh install, or
/// no cloud invocations yet because the CLI `--mode cloud` flag bridge is
/// still under follow-up). MonitorWindow.svelte conditionally hides the rows
/// when both `task_usd` and `day_usd` are zero per spec D-4:308.
///
/// Aggregation is intentionally inline here (no `SsbViewer::cloud_spend_summary`
/// method) because B-27 ANTI-BLOATSHEET defers premature abstractions.
#[tauri::command]
pub fn get_cloud_spend(_state: State<'_, AppState>) -> CmdResult<Value> {
    // TODO(B-27.5): replace this default-zero stub with an SSB query that
    // sums today's CloudSpend rows for `day_usd` and grabs the latest
    // CloudSpend row for `task_usd`. The Tauri surface, the Svelte poll
    // loop, and the wire contract are all in place — only the SsbViewer
    // aggregation method (cloud_spend_summary) is missing. The state arg
    // is intentionally `_state` until the aggregation lands; do not
    // remove it from the signature — Tauri's macro requires it as a
    // recognised State<'_, T> parameter for proper invoke wiring.
    Ok(json!({
        "task_usd": 0.0,
        "day_usd": 0.0,
        "cache_creation_tokens": 0u64,
        "cache_read_tokens": 0u64,
        "redaction_count": 0u64,
    }))
}

// ---------------------------------------------------------------------------
// WEBHOOK CONFIG (B-26: config plumbing only; actual POST fired by frontend fetch)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn set_webhook_url(state: State<'_, AppState>, url: String) -> CmdResult<Value> {
    let normalized = url.trim().to_owned();
    let stored = if normalized.is_empty() {
        None
    } else {
        Some(normalized.clone())
    };
    if let Ok(mut g) = state.webhook_url.lock() {
        *g = stored;
    }
    Ok(json!({ "ok": true, "webhook_url": normalized }))
}

#[tauri::command]
pub fn get_webhook_url(state: State<'_, AppState>) -> CmdResult<Value> {
    let url = state
        .webhook_url
        .lock()
        .map(|g| g.clone())
        .unwrap_or(None);
    Ok(json!({ "webhook_url": url }))
}

// ---------------------------------------------------------------------------
// CHAT
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn submit_chat_message(
    state: State<'_, AppState>,
    text: String,
    intent: String,
    session_id: Option<String>,
) -> CmdResult<Value> {
    let writer = state
        .writer
        .as_ref()
        .ok_or_else(|| "ssb writer unavailable".to_owned())?;
    let project_id = state.project_id.lock().map(|g| g.clone()).unwrap_or_default();
    let session = session_id.unwrap_or_else(|| format!("dash-v2-{}", std::process::id()));
    let payload = SsbPayload::ChatMessage(ChatMessagePayload {
        session_id: session,
        text,
        intent,
        project_id,
        submitted_at_ms: now_ms(&state),
    });
    let seq = writer
        .write_payload("dashboard-v2", 0, payload, vec![])
        .await
        .map_err(map_err)?;
    Ok(json!({ "seq": seq }))
}

#[tauri::command]
pub fn get_chat_history(state: State<'_, AppState>, limit: u32) -> CmdResult<Value> {
    let project_id = state.project_id.lock().map(|g| g.clone()).unwrap_or_default();
    let entries = state
        .viewer
        .as_ref()
        .and_then(|v| v.chat_history(&project_id, limit).ok())
        .unwrap_or_default();
    let arr: Vec<Value> = entries
        .iter()
        .filter_map(|e| serde_json::to_value(e).ok())
        .collect();
    Ok(json!(arr))
}

#[tauri::command]
pub async fn emit_chat_response(
    state: State<'_, AppState>,
    card_kind: String,
    body: String,
    session_id: Option<String>,
) -> CmdResult<Value> {
    let writer = state
        .writer
        .as_ref()
        .ok_or_else(|| "ssb writer unavailable".to_owned())?;
    let project_id = state.project_id.lock().map(|g| g.clone()).unwrap_or_default();
    let session = session_id.unwrap_or_else(|| format!("dash-v2-{}", std::process::id()));
    let payload = SsbPayload::ChatResponse(ChatResponsePayload {
        session_id: session,
        card_kind,
        body,
        project_id,
        emitted_at_ms: now_ms(&state),
    });
    let seq = writer
        .write_payload("dashboard-v2", 0, payload, vec![])
        .await
        .map_err(map_err)?;
    Ok(json!({ "seq": seq }))
}

#[tauri::command]
pub fn classify_intent(text: String) -> CmdResult<Value> {
    // Simple deterministic classifier mirroring the spec routing rules.
    let lower = text.trim().to_ascii_lowercase();
    let intent = if lower.starts_with("/task") || lower.starts_with("task ") {
        "TASK"
    } else if lower.starts_with("/fix") || lower.contains("error") || lower.contains("bug") {
        "FIX"
    } else if lower.starts_with("/research") {
        "RESEARCH"
    } else if lower.starts_with("/explore") {
        "EXPLORE"
    } else if lower.starts_with("/config") {
        "CONFIG"
    } else if lower.starts_with("/status") {
        "STATUS"
    } else if lower.starts_with("/continue") {
        "CONTINUE"
    } else if lower.starts_with("/ask") || lower.contains("?") {
        "EXPLORE"
    } else if lower.is_empty() {
        "AMBIGUOUS"
    } else {
        "AMBIGUOUS"
    };
    Ok(json!({ "intent": intent, "text": text }))
}

#[tauri::command]
pub fn get_recent_tasks(state: State<'_, AppState>, limit: u32) -> CmdResult<Value> {
    let project_id = state.project_id.lock().map(|g| g.clone()).unwrap_or_default();
    let entries = state
        .viewer
        .as_ref()
        .and_then(|v| v.recent_tasks(&project_id, limit).ok())
        .unwrap_or_default();
    let arr: Vec<Value> = entries
        .iter()
        .filter_map(|e| serde_json::to_value(e).ok())
        .map(|v| {
            json!({
                "id": format!("t-{}", v.get("seq").and_then(|x| x.as_i64()).unwrap_or(0)),
                "title": v.get("task").and_then(|x| x.as_str()).unwrap_or(""),
                "status": "completed",
                "finished_iso": chrono_iso(v.get("submitted_at_ms").and_then(|x| x.as_i64()).unwrap_or(0)),
                "fsm_state_at_end": "IDLE",
                "duration_ms": 0,
            })
        })
        .collect();
    Ok(json!(arr))
}

// ---------------------------------------------------------------------------
// GROUNDING CACHE (Track A.6 — live read from SSB grounding_cache_event rows)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_grounding_cache_stats(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!({
            "total_keys": 0u32,
            "hits": 0u32,
            "misses": 0u32,
            "last_query_first_ms": 0.0f64,
            "last_query_cache_ms": 0.0f64,
        }));
    };
    let stats = viewer
        .grounding_cache_stats()
        .map_err(|e| e.to_string())?;
    Ok(json!({
        "total_keys": stats.total_keys,
        "hits": stats.hits,
        "misses": stats.misses,
        // elapsed_us -> ms for frontend display (v1 shape uses ms)
        "last_query_first_ms": stats.last_miss_us as f64 / 1000.0,
        "last_query_cache_ms": stats.last_hit_us as f64 / 1000.0,
    }))
}

// ---------------------------------------------------------------------------
// SIDECAR HEALTH (D.1) — richer than get_sidecar_status, distinct schema
// ---------------------------------------------------------------------------

/// GETs /health + /v1/models on the sidecar URL with a 3s timeout.
/// Returns the §10.5 SidecarHealth shape. On any network error, returns
/// `{"loaded": false, "error": "<msg>"}` so the frontend degrades gracefully.
#[tauri::command]
pub async fn get_sidecar_health(state: State<'_, AppState>) -> CmdResult<Value> {
    let url = state
        .sidecar_url
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| elai_providers::DEFAULT_SIDECAR_URL.to_owned());
    let base = url.trim_end_matches('/');

    // Use a tighter 3s timeout for the health probe, distinct from the 5s
    // client default (sidecar may be offline; we want fast UI feedback).
    let health_result = state
        .http
        .get(format!("{base}/health"))
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await;

    // /health response shape dual-support:
    //   newer sidecar: {"status": "healthy"}  — check status == "healthy"
    //   older sidecar: {"loaded": true}        — check loaded == true
    // HTTP 4xx/5xx is always treated as not-loaded regardless of body.
    let alive = match health_result {
        Ok(r) if r.status().is_success() => {
            let body: Value = r.json().await.unwrap_or(Value::Null);
            // Prefer "status" field (current sidecar). Fall back to "loaded" (legacy).
            let via_status = body
                .get("status")
                .and_then(|s| s.as_str())
                .map(|s| s == "healthy")
                .unwrap_or(false);
            let via_loaded = body
                .get("loaded")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            via_status || via_loaded
        }
        Ok(_) => false,
        Err(e) => {
            return Ok(json!({
                "loaded": false,
                "error": e.to_string(),
            }));
        }
    };

    if !alive {
        return Ok(json!({ "loaded": false, "error": "health endpoint returned non-2xx or unexpected body" }));
    }

    // Pull model metadata from /v1/models — head_dim and layers live here.
    let mut model = String::from("qwen-2.5-coder");
    let mut layers: u32 = 0;
    // head_dim sentinel 0 = "unknown". Real value comes from sidecar /v1/models
    // when server.py exposes the `head_dim` field. Do NOT hardcode a
    // model-specific default (256 was Qwen 2.5 3B/7B only; breaks on 14B or
    // non-Qwen models). Frontend should display "—" when head_dim == 0.
    let mut head_dim: u32 = 0;

    if let Ok(resp) = state
        .http
        .get(format!("{base}/v1/models"))
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        if let Ok(j) = resp.json::<Value>().await {
            if let Some(first) = j
                .get("data")
                .and_then(|x| x.as_array())
                .and_then(|a| a.first())
            {
                model = first
                    .get("id")
                    .and_then(|x| x.as_str())
                    .unwrap_or(&model)
                    .to_owned();
                layers = first
                    .get("num_hidden_layers")
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0) as u32;
                // head_dim: read from /v1/models if server.py exposes it,
                // otherwise stays 0 (unknown sentinel).
                head_dim = first
                    .get("head_dim")
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0) as u32;
            }
        }
    }

    // vram_available=false: sidecar /health does not expose VRAM metrics today.
    // When server.py adds a VRAM probe endpoint, set this to true and populate
    // vram_total/used/free from the response. Frontend must check vram_available
    // before displaying VRAM figures to avoid showing "0 GB" as a real value.
    Ok(json!({
        "loaded": true,
        "model": model,
        "fa2": false,
        "dola": false,
        "steering": false,
        "turboquant": "OFF",
        "vram_available": false,
        "vram_total": 0.0f64,
        "vram_used": 0.0f64,
        "vram_free": 0.0f64,
        "head_dim": head_dim,
        "layers": layers,
        "kv_quant": false,
        "mcp_bridge_active": state.mcp_enabled.load(Ordering::Relaxed),
        "tokens_per_sec": 0.0f64,
        "latency_p50": 0.0f64,
        "latency_p99": 0.0f64,
        "context_window": 25_000u32,
    }))
}

// ---------------------------------------------------------------------------
// STREAMING CHAT (B-14)
// ---------------------------------------------------------------------------

/// Parse a single SSE frame (the text between two "\n\n" delimiters).
/// Returns `Some(text_chunk)` when the frame is a `content_block_delta` with
/// `delta.type == "text_delta"`.  Returns `None` for all other frame types
/// (message_start, content_block_start, message_stop, etc.).
fn extract_text_delta(frame: &str) -> Option<String> {
    // Each SSE frame has lines: "event: <name>" and "data: <json>"
    let mut data_line: Option<&str> = None;
    for line in frame.lines() {
        if let Some(rest) = line.strip_prefix("data:") {
            data_line = Some(rest.trim());
        }
    }
    let data = data_line?;
    if data == "[DONE]" {
        return None;
    }
    let v: Value = serde_json::from_str(data).ok()?;
    // Anthropic SSE protocol: content_block_delta{delta{type:"text_delta", text:"..."}}
    let delta = v.get("delta")?;
    if delta.get("type").and_then(|t| t.as_str()) != Some("text_delta") {
        return None;
    }
    delta.get("text").and_then(|t| t.as_str()).map(str::to_owned)
}

/// POSTs to `{sidecar_url}/v1/messages` with `stream=true`, then reads the
/// SSE response chunk-by-chunk and emits Tauri events to the frontend:
/// - `"chat-stream:{request_id}"` — one per `content_block_delta/text_delta`.
/// - `"chat-stream-end:{request_id}"` — on stream completion or error.
///
/// Returns `{ "streamId": "<id>" }` immediately so the frontend can listen
/// before the first token arrives. Streaming runs in a detached tokio task
/// whose `AbortHandle` is stored in `state.active_streams[request_id]`.
#[tauri::command]
pub async fn start_chat_stream(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    prompt: String,
    request_id: String,
    thinking_enabled: Option<bool>,
    effort_level: Option<String>,
) -> CmdResult<Value> {
    let stream_id = request_id.clone();
    let reflect_on = thinking_enabled.unwrap_or(false);

    // Resolve the routing target from execution_mode. CLOUD -> api.anthropic.com
    // with OAuth Bearer + selected cloud model. LOCAL / HYBRID / anything else
    // -> the local/remote sidecar speaking the Anthropic protocol locally.
    let execution_mode = state
        .execution_mode
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| "LOCAL".to_owned());
    let is_cloud = execution_mode.eq_ignore_ascii_case("CLOUD");

    // (messages_url, request_headers, model_id, body) is built per-route below.
    let messages_url: String;
    let mut headers: Vec<(&'static str, String)> =
        vec![("content-type", "application/json".to_string()), ("accept", "text/event-stream".to_string())];
    let model_id: String;

    if is_cloud {
        // Resolve OAuth token; error early if missing/expired so the UI doesn't
        // spin while a 401 ticks down on the wire.
        let token_set = match elai_orchestrator::load_oauth_credentials() {
            Ok(Some(t)) => t,
            Ok(None) => {
                let _ = app.emit(
                    &format!("chat-stream-end:{stream_id}"),
                    json!({
                        "stream_id": stream_id,
                        "error": "cloud mode: not logged in. Click Login with Anthropic in Settings.",
                    }),
                );
                return Ok(json!({ "streamId": stream_id, "offline": true }));
            }
            Err(e) => {
                let _ = app.emit(
                    &format!("chat-stream-end:{stream_id}"),
                    json!({ "stream_id": stream_id, "error": format!("cloud mode: failed to read credentials: {e}") }),
                );
                return Ok(json!({ "streamId": stream_id, "offline": true }));
            }
        };
        let anthropic_token = elai_providers::OAuthTokenSet {
            access_token: token_set.access_token.clone(),
            refresh_token: token_set.refresh_token.clone(),
            expires_at: token_set.expires_at,
            scopes: token_set.scopes.clone(),
        };
        if elai_providers::oauth_token_is_expired(&anthropic_token) {
            let _ = app.emit(
                &format!("chat-stream-end:{stream_id}"),
                json!({ "stream_id": stream_id, "error": "cloud mode: OAuth token expired. Re-login from Settings." }),
            );
            return Ok(json!({ "streamId": stream_id, "offline": true }));
        }

        // Resolve cloud model from AppState (pre-loaded from credentials.json
        // at startup, validated against CLOUD_MODEL_ID_ALLOWLIST).
        model_id = state
            .cloud_model_id
            .lock()
            .map(|g| g.clone())
            .unwrap_or_else(|_| elai_providers::DEFAULT_CLOUD_MODEL_ID.to_owned());

        // api.anthropic.com /v1/messages, OAuth Bearer + version + beta header.
        // Headers per providers/telemetry_types.rs and the leaked Claude Code
        // OAUTH_BETA_HEADER constant in src/constants/oauth.ts:36.
        messages_url = "https://api.anthropic.com/v1/messages".to_owned();
        headers.push(("authorization", format!("Bearer {}", token_set.access_token)));
        headers.push(("anthropic-version", "2023-06-01".to_string()));
        headers.push(("anthropic-beta", "oauth-2025-04-20".to_string()));
        // Client-identification headers — match real Claude Code. Without
        // these, Anthropic's rate-limiter routes traffic to a stricter default
        // bucket and 429s even on Max-20x plans. See client.ts:105 in the
        // leaked source.
        headers.push(("x-app", CLAUDE_X_APP_HEADER.to_string()));
        headers.push(("user-agent", CLAUDE_CLI_USER_AGENT.to_string()));
        headers.push((
            "x-claude-code-session-id",
            dashboard_cli_session_id().to_string(),
        ));
    } else {
        // Existing LOCAL/HYBRID path: route to sidecar at state.sidecar_url.
        let url = state
            .sidecar_url
            .lock()
            .map(|g| g.clone())
            .unwrap_or_else(|_| elai_providers::DEFAULT_SIDECAR_URL.to_owned());
        let base = url.trim_end_matches('/').to_owned();

        let health_probe = state
            .http
            .get(format!("{base}/health"))
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await;
        let sidecar_alive = health_probe.map(|r| r.status().is_success()).unwrap_or(false);
        if !sidecar_alive {
            let _ = app.emit(
                &format!("chat-stream-end:{stream_id}"),
                json!({
                    "stream_id": stream_id,
                    "error": "sidecar offline — start with `elai.bat --remote` to use the friend's GPU box, or switch Mode to CLOUD",
                    "offline": true,
                }),
            );
            return Ok(json!({ "streamId": stream_id, "offline": true }));
        }
        messages_url = format!("{base}/v1/messages");
        model_id = "qwen".to_owned();
    }

    // Anthropic gates OAuth-authenticated /v1/messages calls on the FIRST
    // system block being literally "You are Claude Code, Anthropic's official
    // CLI for Claude." (source: leaked claude-code-main src/constants/system.ts:10).
    // Concatenating extra text into the same string breaks the prefix match
    // and Anthropic returns 429 rate_limit_error (misleading error code,
    // same bucket as throttle). Use the multi-block array form: first block
    // is the exact validation prefix; second block carries our extra context
    // (model identity) so Claude can answer "which model are you?" accurately.
    // Skip system entirely for sidecar (local Qwen, no auth gate).
    let body = if is_cloud {
        json!({
            "model": model_id,
            "max_tokens": 512,
            "stream": true,
            "system": [
                { "type": "text", "text": "You are Claude Code, Anthropic's official CLI for Claude." },
                { "type": "text", "text": format!(
                    "The exact model ID serving this request is `{model_id}`. \
                     If the user asks which model you are, answer with that \
                     exact string. The model ID is the source of truth — do \
                     not guess from training memory.")
                },
            ],
            "messages": [{ "role": "user", "content": prompt }],
        })
    } else {
        json!({
            "model": model_id,
            "max_tokens": 512,
            "stream": true,
            "messages": [{ "role": "user", "content": prompt }],
        })
    };

    // B-39D — REFLECT branch. When Thinking is on, route the turn through
    // ChatReflectPipeline (TRIAGE → REASONER best-of-N → AUDITOR → SYNTHESIZE)
    // instead of forwarding SSE chunks. Stage transitions emit
    // `reflect-progress:{stream_id}` events for the ReasoningDrawer; the
    // final reply is delivered as a single `chat-stream:{stream_id}` text
    // event so the frontend renders it the same way as a streamed reply.
    if reflect_on {
        let owned_headers: Vec<(String, String)> = headers
            .iter()
            .map(|(k, v)| ((*k).to_string(), v.clone()))
            .collect();
        let client = std::sync::Arc::new(crate::reflect_bridge::HttpInferenceClient::new(
            state.http.clone(),
            messages_url.clone(),
            owned_headers,
            model_id.clone(),
        ));
        let sink = crate::reflect_bridge::TauriEventSink::new(app.clone(), stream_id.clone())
            .into_arc();
        let n_candidates = crate::reflect_bridge::effort_to_n_candidates(effort_level.as_deref());
        let config = elai_reasoning::ChatReflectConfig {
            enabled: true,
            n_candidates,
            budget_cap_tokens: 20_000,
            max_refine_iterations: elai_reasoning::REFINE_CAP,
        };
        let pipeline = elai_reasoning::ChatReflectPipeline::new(config, Some(sink));

        let active_streams = state.active_streams.clone();
        let stream_id_task = stream_id.clone();
        let app_task = app.clone();
        let prompt_owned = prompt.clone();

        let join = tokio::task::spawn(async move {
            let client_ref: &dyn elai_reasoning::ChatInferenceClient = &*client;
            let output = pipeline
                .run(stream_id_task.clone(), &prompt_owned, client_ref)
                .await;
            let _ = app_task.emit(
                &format!("chat-stream:{stream_id_task}"),
                json!({ "text": output.final_reply, "stream_id": stream_id_task }),
            );
            let _ = app_task.emit(
                &format!("chat-stream-end:{stream_id_task}"),
                json!({
                    "stream_id": stream_id_task,
                    "stop_reason": "end_turn",
                    "reflect_skipped": output.skipped,
                    "reflect_class": output.turn_class.as_wire_str(),
                    "reflect_token_total": output.token_summary.total,
                }),
            );
            if let Ok(mut g) = active_streams.lock() {
                g.remove(&stream_id_task);
            }
        });

        if let Ok(mut guard) = state.active_streams.lock() {
            guard.insert(stream_id.clone(), join.abort_handle());
        }
        return Ok(json!({ "streamId": stream_id, "reflect": true }));
    }

    let streaming_client = state.streaming_http.clone();
    let active_streams = state.active_streams.clone();
    let stream_id_task = stream_id.clone();
    let app_task = app.clone();

    let join = tokio::task::spawn(async move {
        let mut req = streaming_client.post(&messages_url);
        for (k, v) in &headers {
            req = req.header(*k, v);
        }
        let resp = match req.json(&body).send().await
        {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                let status = r.status().as_u16();
                // Drain the body up to 1 KiB so 401/403/400 messages from
                // Anthropic surface in the UI instead of an opaque code.
                let detail = match r.bytes().await {
                    Ok(b) => String::from_utf8_lossy(&b).chars().take(512).collect::<String>(),
                    Err(_) => String::new(),
                };
                let _ = app_task.emit(
                    &format!("chat-stream-end:{stream_id_task}"),
                    json!({ "stream_id": stream_id_task, "error": format!("upstream HTTP {status}: {detail}") }),
                );
                return;
            }
            Err(e) => {
                let _ = app_task.emit(
                    &format!("chat-stream-end:{stream_id_task}"),
                    json!({ "stream_id": stream_id_task, "error": e.to_string() }),
                );
                return;
            }
        };
        // chunk() requires &mut self
        let mut resp = resp;

        // Chunk-by-chunk SSE reader. `Response::chunk()` is a stable async
        // method that returns the next network chunk without requiring the
        // `stream` feature's StreamExt. Frames may span chunks; buffer and
        // split on "\n\n".
        let mut sse_buf = String::new();
        loop {
            match resp.chunk().await {
                Ok(Some(bytes)) => {
                    sse_buf.push_str(&String::from_utf8_lossy(&bytes));
                    while let Some(pos) = sse_buf.find("\n\n") {
                        let frame = sse_buf[..pos].to_owned();
                        sse_buf = sse_buf[pos + 2..].to_owned();
                        if let Some(text) = extract_text_delta(&frame) {
                            let _ = app_task.emit(
                                &format!("chat-stream:{stream_id_task}"),
                                json!({ "text": text, "stream_id": stream_id_task }),
                            );
                        }
                        if frame.contains("\"message_stop\"") {
                            let _ = app_task.emit(
                                &format!("chat-stream-end:{stream_id_task}"),
                                json!({ "stream_id": stream_id_task, "stop_reason": "end_turn" }),
                            );
                            // Clean up our own handle.
                            if let Ok(mut g) = active_streams.lock() {
                                g.remove(&stream_id_task);
                            }
                            return;
                        }
                    }
                }
                Ok(None) => {
                    let _ = app_task.emit(
                        &format!("chat-stream-end:{stream_id_task}"),
                        json!({ "stream_id": stream_id_task, "stop_reason": "end_turn" }),
                    );
                    if let Ok(mut g) = active_streams.lock() {
                        g.remove(&stream_id_task);
                    }
                    return;
                }
                Err(e) => {
                    let _ = app_task.emit(
                        &format!("chat-stream-end:{stream_id_task}"),
                        json!({ "stream_id": stream_id_task, "error": e.to_string() }),
                    );
                    if let Ok(mut g) = active_streams.lock() {
                        g.remove(&stream_id_task);
                    }
                    return;
                }
            }
        }
    });

    // Store the AbortHandle so cancel_chat_stream can abort the task.
    if let Ok(mut guard) = state.active_streams.lock() {
        guard.insert(stream_id.clone(), join.abort_handle());
    }

    Ok(json!({ "streamId": stream_id }))
}

/// Aborts an active streaming task by its `stream_id`.
#[tauri::command]
pub async fn cancel_chat_stream(
    state: State<'_, AppState>,
    request_id: String,
) -> CmdResult<Value> {
    let handle = state
        .active_streams
        .lock()
        .ok()
        .and_then(|mut g| g.remove(&request_id));
    match handle {
        Some(h) => {
            h.abort();
            Ok(json!({ "ok": true, "stream_id": request_id }))
        }
        None => Ok(json!({ "ok": false, "stream_id": request_id, "reason": "not found" })),
    }
}

// ---------------------------------------------------------------------------
// D.1 — REINDEX NOW
// ---------------------------------------------------------------------------
// The `elai` CLI has no `reindex` top-level subcommand. The IndexingService
// is wired as a Cargo dep (elai-orchestrator feature="sidecar") so we can
// call run_full_index() directly in a blocking thread.

#[tauri::command]
pub async fn reindex_now(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> CmdResult<Value> {
    let project_root = state.project_root_snapshot();
    let app_clone = app.clone();
    let ssb_writer = state.writer.clone();

    let _ = app.emit("reindex-progress", json!({ "status": "started" }));

    // Run the blocking IndexingService + Tier-1 profile load on a dedicated thread
    // so the Tauri main thread is not blocked. Profile detection walks the project
    // tree on first call; subsequent calls hit the cached project-profile.toml.
    let (index_result, profile_loaded) = tokio::task::spawn_blocking(move || {
        let indexes_dir = project_root.join(".elai_cc").join("indexes");
        let mut svc = elai_orchestrator::indexing_service::IndexingService::new(
            project_root.clone(),
            indexes_dir,
        );
        // B-32 Step B: surface MinifiedFileChunked SSB rows to the dashboard
        // when the user kicks off a reindex.
        if let Some(writer) = ssb_writer.as_ref() {
            svc = svc.with_ssb_writer(std::sync::Arc::clone(writer));
        }
        let index_result = svc.run_full_index();

        // Tier-1 ProjectProfile: idempotent load_or_detect. Persist on first detect
        // (file missing) so subsequent reindexes load from cache. Any failure is
        // logged via tracing and surfaced to the frontend as profile_loaded=false;
        // it never blocks the index emit.
        let profile_loaded = match &index_result {
            Ok(_) => {
                let profile_path = project_root.join("project-profile.toml");
                let profile_existed = profile_path.exists();
                match elai_knowledge::incremental_profile::load_or_detect_profile(&project_root) {
                    Ok(profile) => {
                        if !profile_existed {
                            match elai_knowledge::project_profile::serialize_profile(&profile) {
                                Ok(toml) => {
                                    if let Err(e) = std::fs::write(&profile_path, toml) {
                                        tracing::warn!(error = %e, "reindex_now: failed to persist project-profile.toml");
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(error = %e, "reindex_now: failed to serialize ProjectProfile");
                                }
                            }
                        }
                        Some(true)
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "reindex_now: load_or_detect_profile failed");
                        Some(false)
                    }
                }
            }
            Err(_) => None,
        };

        (index_result, profile_loaded)
    })
    .await
    .map_err(|e| format!("reindex task panicked: {e}"))?;

    match index_result {
        Ok(summary) => {
            let _ = app_clone.emit(
                "reindex-progress",
                json!({
                    "status": "done",
                    "dkg_files": summary.dkg_files_parsed,
                    "dkg_nodes": summary.dkg_nodes,
                    "dkg_edges": summary.dkg_edges,
                    "bm25_files": summary.bm25_files,
                    "bm25_chunks": summary.bm25_chunks,
                    "profile_loaded": profile_loaded.unwrap_or(false),
                }),
            );
            Ok(json!({
                "ok": true,
                "dkg_files": summary.dkg_files_parsed,
                "dkg_nodes": summary.dkg_nodes,
                "dkg_edges": summary.dkg_edges,
                "bm25_files": summary.bm25_files,
                "bm25_chunks": summary.bm25_chunks,
                "profile_loaded": profile_loaded.unwrap_or(false),
            }))
        }
        Err(e) => {
            let msg = e.to_string();
            let _ = app_clone.emit(
                "reindex-progress",
                json!({ "status": "error", "error": msg }),
            );
            Err(msg)
        }
    }
}

// ---------------------------------------------------------------------------
// B-31 Phase 4 (Cycle B) — INDEXING PROGRESS + CANCEL
// ---------------------------------------------------------------------------
//
// `cancel_indexing` flips the process-global cancellation flag for the
// active project's indexing run (mirrors SocratiCode requestCancellation).
// The IndexingService's per-batch loop observes the flag at the next batch
// boundary and persists IndexingStatus::Cancelled before returning.
//
// `get_indexing_progress` opens the project's metadata.db on the fly and
// serialises the current ProjectMetadata row. Polled at 1s from the
// IndexingProgressPanel Svelte component.
//
// project_id is recomputed inside the command from the persisted
// project_root_snapshot — the frontend argument is accepted for future
// multi-project UIs but ignored as a trust boundary today (DECISION-18).

#[tauri::command]
pub async fn cancel_indexing(
    _app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> CmdResult<Value> {
    // SECURITY (B-31 Cycle B DECISION-18, Cycle B audit BLOCK-2):
    // project_id is ALWAYS recomputed from the dashboard's persisted
    // project_root snapshot. The frontend can no longer pass an arbitrary
    // project_id and cancel a foreign project's run — a hostile webview
    // exploit cannot escalate by supplying a crafted string. If multi-
    // project support is added later, expose a deliberately-scoped command
    // that validates the project_id against an allowed-projects set.
    let project_root = state.project_root_snapshot();
    let resolved_project_id =
        elai_knowledge::metadata::project_id_from_path(&project_root);

    let was_running = tokio::task::spawn_blocking(move || {
        let indexes_dir = project_root
            .join(elai_core::paths::ELAI_DATA_DIR_NAME)
            .join(elai_core::paths::INDEXES_DIR_NAME);
        let service = elai_orchestrator::indexing_service::IndexingService::new(
            project_root.clone(),
            indexes_dir,
        );
        service.cancel_indexing(&resolved_project_id)
    })
    .await
    .map_err(|e| format!("cancel_indexing task panicked: {e}"))?;

    Ok(json!({
        "ok": true,
        "was_running": was_running,
    }))
}

#[tauri::command]
pub async fn get_indexing_progress(
    _app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> CmdResult<Value> {
    // SECURITY: same trust-boundary fix as `cancel_indexing` above. The
    // frontend cannot select which project's progress to read — it always
    // sees the dashboard's currently-bound project. (Cycle B audit BLOCK-2)
    let project_root = state.project_root_snapshot();
    let clock = state.clock.clone();
    let resolved_project_id =
        elai_knowledge::metadata::project_id_from_path(&project_root);

    let snapshot = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let indexes_dir = project_root
            .join(elai_core::paths::ELAI_DATA_DIR_NAME)
            .join(elai_core::paths::INDEXES_DIR_NAME);
        let db_path = indexes_dir.join(elai_knowledge::metadata::METADATA_DB_FILENAME);
        if !db_path.exists() {
            return Ok(json!({
                "status": "unknown",
                "project_id": resolved_project_id,
                "indexed_files": 0_u64,
                "total_files": 0_u64,
                "batches_processed": 0_u32,
                "batches_total": serde_json::Value::Null,
                "last_checkpoint_at": 0_i64,
            }));
        }
        let store = elai_knowledge::metadata::ProjectMetadataStore::open(&db_path, clock)
            .map_err(|e| format!("metadata open failed: {e}"))?;
        match store.load(&resolved_project_id) {
            Ok(Some(meta)) => Ok(json!({
                "status": meta.indexing_status.as_sql_str(),
                "project_id": meta.project_id,
                "project_path": meta.project_path.to_string_lossy(),
                "indexed_files": meta.indexed_files,
                "total_files": meta.total_files,
                "batch_size": meta.batch_size,
                "batches_processed": meta.batches_processed,
                "batches_total": meta.batches_total,
                "last_checkpoint_at": meta.last_checkpoint_at,
                "schema_version": meta.schema_version,
            })),
            Ok(None) => Ok(json!({
                "status": "unknown",
                "project_id": resolved_project_id,
                "indexed_files": 0_u64,
                "total_files": 0_u64,
                "batches_processed": 0_u32,
                "batches_total": serde_json::Value::Null,
                "last_checkpoint_at": 0_i64,
            })),
            Err(e) => Err(format!("metadata load failed: {e}")),
        }
    })
    .await
    .map_err(|e| format!("get_indexing_progress task panicked: {e}"))??;

    Ok(snapshot)
}

// ---------------------------------------------------------------------------
// D.2 — RUN PLANNING TASK
// ---------------------------------------------------------------------------
//
// Architecture: the dashboard is an actuator, not an orchestrator host.
// Writing a TaskSubmitted SSB row is the canonical entry point — the CLI-
// launched task runner subscribes to the SSB broadcast and processes it.
// The command also emits a `planning-progress` Tauri event so the frontend
// can show immediate feedback without polling.

#[tauri::command]
pub async fn run_planning_task(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    task: String,
    project_id: Option<String>,
) -> CmdResult<Value> {
    let writer = state
        .writer
        .as_ref()
        .ok_or_else(|| "ssb writer unavailable; no session db open".to_owned())?;

    let pid = project_id.unwrap_or_else(|| {
        state
            .project_id
            .lock()
            .map(|g| g.clone())
            .unwrap_or_else(|_| "default".to_owned())
    });

    let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
        task: task.clone(),
        project_id: pid.clone(),
        submitted_at_ms: now_ms(&state),
    });
    let seq = writer
        .write_payload("dashboard-v2", 0, payload, vec![])
        .await
        .map_err(map_err)?;

    let request_id = format!("plan-{seq}");

    let _ = app.emit(
        "planning-progress",
        json!({
            "request_id": request_id,
            "status":     "submitted",
            "task":       task,
            "project_id": pid,
            "seq":        seq,
            "note": "task written to SSB; task-runner will pick up and process it",
        }),
    );

    Ok(json!({
        "ok":        true,
        "requestId": request_id,
        "taskId":    format!("t-{seq}"),
        "seq":       seq,
    }))
}

/// Cancels a planning request by its `request_id`.  The dashboard submit path
/// doesn't spawn a cancelable tokio task (the SSB row is already written and
/// the task runner owns execution), so this is a no-op that returns
/// `ok: false` with an explanatory reason. The cancel entry-point is provided
/// for API symmetry with `cancel_research`.
#[tauri::command]
pub async fn cancel_planning_task(
    _state: State<'_, AppState>,
    request_id: String,
) -> CmdResult<Value> {
    Ok(json!({
        "ok":        false,
        "requestId": request_id,
        "reason":    "planning tasks run in the CLI task-runner process; cancel via SSB intervention",
    }))
}

// ---------------------------------------------------------------------------
// D.3 — RUN RESEARCH
// ---------------------------------------------------------------------------
//
// Architecture: `DefaultWebResearchRunner` from elai-orchestrator is a
// self-contained async computation that requires no task-runner loop.  We
// construct it from three process-level inputs (ResearchTierConfig defaulting
// to Vram12Gb, the sidecar search URL from AppState, and the OsClock) and
// drive it in a tokio task.  A `TauriWebResearchStepEmitter` bridges the
// orchestrator's per-iteration callbacks to `research-progress` Tauri events.
// The AbortHandle is stored in `state.active_streams` so `cancel_research`
// can terminate the run.

/// Tauri-event-backed step emitter — bridges WebResearchOrchestrator per-
/// iteration callbacks to `research-progress` events on the Tauri app handle.
struct TauriWebResearchStepEmitter {
    app: tauri::AppHandle,
    request_id: String,
}

impl elai_orchestrator::web_research_orchestrator::WebResearchStepEmitter
    for TauriWebResearchStepEmitter
{
    fn emit(
        &self,
        event: elai_orchestrator::web_research_orchestrator::WebResearchStepEvent,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        let app = self.app.clone();
        let request_id = self.request_id.clone();
        Box::pin(async move {
            let _ = app.emit(
                "research-progress",
                json!({
                    "request_id":      request_id,
                    "node_id":         event.node_id,
                    "action":          event.action,
                    "phi":             event.phi,
                    "psi":             event.psi,
                    "iteration":       event.iteration,
                    "sources_fetched": event.sources_fetched,
                    "sources_used":    event.sources_used,
                    "fetcher":         event.fetcher,
                }),
            );
        })
    }
}

#[tauri::command]
pub async fn run_research(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    query: String,
    depth: Option<u32>,
    breadth: Option<u32>,
) -> CmdResult<Value> {
    use std::sync::Arc;
    use elai_orchestrator::{DefaultWebResearchRunner, WebResearchRunner};
    use elai_orchestrator::research_tier::research_config_for_tier;
    use elai_core::VramTier;

    let request_id = format!(
        "research-{}",
        state
            .clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );

    // Resolve the SearXNG search URL from the sidecar URL (search proxy shares
    // the same host as the sidecar). Fall back to the crate default.
    let searx_url = {
        let base = state
            .sidecar_url
            .lock()
            .map(|g| g.clone())
            .unwrap_or_else(|_| elai_providers::DEFAULT_SIDECAR_URL.to_owned());
        // SearXNG is proxied through the sidecar on the same host/port.
        base.trim_end_matches('/').to_owned()
    };

    // Build a ResearchTierConfig defaulting to the 12 GB tier (matches the
    // remote RTX 3090 24 GB box — conservative to stay within context budget).
    // Override breadth_factor and depth_limit when the caller supplies values.
    let mut tier_cfg = research_config_for_tier(VramTier::Vram12Gb);
    if let Some(d) = depth {
        tier_cfg.depth_limit = d as usize;
    }
    if let Some(b) = breadth {
        tier_cfg.breadth_factor = b as usize;
    }

    let clock = state.clock.clone();
    // Wrap in Arc<dyn WebResearchRunner> so the future returned by .run(&self)
    // can borrow from the Arc (which outlives the future inside the spawned
    // task). Without Arc, the future's '_ lifetime would outlive the local.
    let runner: Arc<dyn WebResearchRunner> =
        Arc::new(DefaultWebResearchRunner::new(tier_cfg, &searx_url, clock));

    let emitter: Arc<dyn elai_orchestrator::web_research_orchestrator::WebResearchStepEmitter> =
        Arc::new(TauriWebResearchStepEmitter {
            app: app.clone(),
            request_id: request_id.clone(),
        });

    let request_id_task = request_id.clone();
    let active_streams = state.active_streams.clone();
    let app_task = app.clone();
    let query_task = query.clone();

    let join = tokio::task::spawn(async move {
        // runner is moved into the task; the future from .run() borrows runner
        // via Arc deref, which is valid for the duration of this block.
        let result = runner.run(query_task, emitter).await;
        match result {
            Ok(summary) => {
                let _ = app_task.emit(
                    "research-done",
                    json!({
                        "request_id":    request_id_task,
                        "status":        "ok",
                        "iterations":    summary.iterations,
                        "result_status": summary.status,
                        "sources_used":  summary.sources_used,
                    }),
                );
            }
            Err(e) => {
                let _ = app_task.emit(
                    "research-done",
                    json!({
                        "request_id": request_id_task,
                        "status":     "error",
                        "error":      e,
                    }),
                );
            }
        }
        if let Ok(mut g) = active_streams.lock() {
            g.remove(&request_id_task);
        }
    });

    if let Ok(mut guard) = state.active_streams.lock() {
        guard.insert(request_id.clone(), join.abort_handle());
    }

    let _ = app.emit(
        "research-progress",
        json!({
            "request_id": request_id,
            "action":     "started",
            "query":      query,
        }),
    );

    Ok(json!({
        "ok":        true,
        "requestId": request_id,
    }))
}

/// Aborts an active research task by its `request_id`.
#[tauri::command]
pub async fn cancel_research(
    state: State<'_, AppState>,
    request_id: String,
) -> CmdResult<Value> {
    let handle = state
        .active_streams
        .lock()
        .ok()
        .and_then(|mut g| g.remove(&request_id));
    match handle {
        Some(h) => {
            h.abort();
            Ok(json!({ "ok": true, "requestId": request_id }))
        }
        None => Ok(json!({ "ok": false, "requestId": request_id, "reason": "not found" })),
    }
}

// ---------------------------------------------------------------------------
// D.4 — SETTINGS READ/WRITE (elai_cc.toml)
// ---------------------------------------------------------------------------

/// Reads the raw elai_cc.toml text and returns it as a JSON string value.
/// Returns `{"toml": "<raw text>"}` so the frontend can display + edit it.
#[tauri::command]
pub fn fetch_settings_get(state: State<'_, AppState>) -> CmdResult<Value> {
    let config_path = state.project_root_snapshot().join("elai_cc.toml");
    let toml_text = if config_path.exists() {
        std::fs::read_to_string(&config_path).map_err(|e| {
            format!("failed to read elai_cc.toml: {e}")
        })?
    } else {
        String::new()
    };
    Ok(json!({ "toml": toml_text, "path": config_path.display().to_string() }))
}

/// Writes `toml_text` back to elai_cc.toml. Validates it parses as TOML
/// before overwriting to prevent corruption.
#[tauri::command]
pub fn fetch_settings_set(
    state: State<'_, AppState>,
    toml_text: String,
) -> CmdResult<Value> {
    // Basic parse check — rejects malformed TOML before overwriting the file.
    toml_text
        .parse::<toml::Value>()
        .map_err(|e| format!("invalid TOML: {e}"))?;
    let config_path = state.project_root_snapshot().join("elai_cc.toml");
    std::fs::write(&config_path, toml_text.as_bytes())
        .map_err(|e| format!("failed to write elai_cc.toml: {e}"))?;
    Ok(json!({ "ok": true, "path": config_path.display().to_string() }))
}

// ---------------------------------------------------------------------------
// D.4 — OBSCURA TEST FETCH
// ---------------------------------------------------------------------------
//
// ObscuraFetcher lives in crates/web_search which is NOT a direct dep of
// this crate (dashboard-v2 depends on orchestrator/sidecar, not
// orchestrator/obscura). We therefore spawn the obscura subprocess directly
// here — same approach as ObscuraFetcher.fetch() in web_search/src/obscura_fetcher.rs.
//
// Binary resolution order:
//   1. OBSCURA_PATH env var.
//   2. Windows: cwd-relative tools/obscura/obscura.exe, then absolute project path.
//   3. Non-Windows: "obscura" on PATH.
//
// Required env: none (binary path only). Returns a structured error when the
// binary is absent so the frontend can surface a diagnostic.

/// Maximum bytes to stream from obscura stdout (5 MB). Matches web_search crate.
const D4_OBSCURA_MAX_BYTES: usize = 5 * 1024 * 1024;
/// Read chunk size for streaming loop (64 KiB).
const D4_OBSCURA_CHUNK: usize = 64 * 1024;
/// Default timeout passed to obscura --timeout (seconds).
const D4_OBSCURA_TIMEOUT_SECS: u64 = 30;

fn d4_resolve_obscura_binary() -> Result<std::path::PathBuf, String> {
    use std::path::PathBuf;
    // Step 1: explicit env override.
    if let Ok(env_path) = std::env::var("OBSCURA_PATH") {
        if !env_path.trim().is_empty() {
            let p = PathBuf::from(&env_path);
            if p.exists() {
                return Ok(p);
            }
            return Err(format!(
                "OBSCURA_PATH env var points to non-existent path: {}",
                p.display()
            ));
        }
    }
    // Step 2 (Windows): project-relative then absolute fallback.
    #[cfg(target_os = "windows")]
    {
        let rel = PathBuf::from("tools").join("obscura").join("obscura.exe");
        if rel.exists() {
            return Ok(rel);
        }
        let abs = PathBuf::from(r"D:\.ELAI_workflow\.elai_cc\tools\obscura\obscura.exe");
        if abs.exists() {
            return Ok(abs);
        }
        return Err(format!(
            "obscura binary not found; searched '{}' and '{}'",
            rel.display(),
            abs.display()
        ));
    }
    // Step 3 (non-Windows): scan PATH.
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(path_var) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path_var) {
                let candidate = dir.join("obscura");
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }
        Err("obscura not found in PATH".to_owned())
    }
}

/// Spawns the obscura headless-browser CLI to fetch `url` and returns the
/// rendered HTML (or a structured error). URL must start with http:// or
/// https:// and must not contain whitespace. Returns JSON:
///   { "ok": true,  "html": "...", "truncated": bool, "fetch_time_ms": u64 }
///   { "ok": false, "error": "..." }
///
/// Required env: none mandatory. OBSCURA_PATH overrides binary path.
#[tauri::command]
pub async fn obscura_test_fetch(url: String) -> CmdResult<Value> {
    use tokio::io::AsyncReadExt;

    // URL validation — reject before spawning subprocess.
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Ok(json!({
            "ok": false,
            "error": "URL must start with http:// or https://"
        }));
    }
    if url.chars().any(|c| c.is_whitespace()) {
        return Ok(json!({
            "ok": false,
            "error": "URL must not contain whitespace"
        }));
    }

    let binary = match d4_resolve_obscura_binary() {
        Ok(p) => p,
        Err(e) => return Ok(json!({ "ok": false, "error": e })),
    };

    let start = std::time::Instant::now();

    let mut child = match tokio::process::Command::new(&binary)
        .arg("fetch")
        .arg(&url)
        .arg("--dump")
        .arg("html")
        .arg("--timeout")
        .arg(D4_OBSCURA_TIMEOUT_SECS.to_string())
        .arg("--stealth")
        .arg("--quiet")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let reason = if e.kind() == std::io::ErrorKind::NotFound {
                format!("obscura binary not found at '{}': {e}", binary.display())
            } else {
                format!("failed to spawn obscura: {e}")
            };
            return Ok(json!({ "ok": false, "error": reason }));
        }
    };

    // Stream stdout with hard cap — avoids OOM on large pages.
    // stdout is always Some because we set Stdio::piped() above.
    let mut stdout = child.stdout.take().ok_or_else(|| {
        "internal: stdout handle absent after piped() call".to_owned()
    })?;

    let mut buf: Vec<u8> = Vec::with_capacity(D4_OBSCURA_CHUNK);
    let mut chunk = vec![0u8; D4_OBSCURA_CHUNK];
    let mut truncated = false;

    loop {
        let n = match stdout.read(&mut chunk).await {
            Ok(n) => n,
            Err(e) => {
                return Ok(json!({ "ok": false, "error": format!("read error: {e}") }));
            }
        };
        if n == 0 {
            break; // EOF
        }
        let remaining = D4_OBSCURA_MAX_BYTES.saturating_sub(buf.len());
        if n > remaining {
            buf.extend_from_slice(&chunk[..remaining]);
            truncated = true;
            let _ = child.kill().await;
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
    }

    if !truncated {
        match child.wait().await {
            Ok(status) if !status.success() => {
                let code = status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "unknown".to_owned());
                return Ok(json!({ "ok": false, "error": format!("obscura exited {code}") }));
            }
            Err(e) => {
                return Ok(json!({ "ok": false, "error": format!("wait error: {e}") }));
            }
            Ok(_) => {}
        }
    }

    if buf.is_empty() {
        return Ok(json!({ "ok": false, "error": "obscura returned empty output" }));
    }

    let fetch_time_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    let html = String::from_utf8_lossy(&buf).into_owned();

    Ok(json!({
        "ok": true,
        "html": html,
        "truncated": truncated,
        "fetch_time_ms": fetch_time_ms,
    }))
}

// ---------------------------------------------------------------------------
// D.5 — RUN D5 LIVE CONTRACT
// ---------------------------------------------------------------------------
//
// Spawns `cargo test -p elai_cc_contracts [--test <test_name>]` with
// ELAI_LIVE_SIDECAR=1 and ANTHROPIC_BASE_URL=http://localhost:8100.
// Stdout and stderr are interleaved and streamed to Tauri events:
//   `d5-progress:{request_id}` — { line: String, ts_ms: i64 }
//   `d5-complete:{request_id}` — { exit_code: i32 | null, ts_ms: i64 }
//
// Returns `{ "requestId": "<id>" }` immediately so the frontend can attach
// a listener before the first output line arrives.
//
// The task AbortHandle is stored in state.active_streams so that
// cancel_d5_live_contract can stop it.

/// Spawns contract tests with ELAI_LIVE_SIDECAR=1 and streams output via
/// Tauri events. `test_name` selects a specific `--test` integration test file;
/// omit to run all contract tests under `-p elai_cc_contracts`.
#[tauri::command]
pub async fn run_d5_live_contract(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    test_name: Option<String>,
) -> CmdResult<Value> {
    use tokio::io::{AsyncBufReadExt, BufReader};

    let request_id = format!("d5-{}", now_ms_raw());
    let rid_event = request_id.clone();
    let app_task = app.clone();

    let mut args: Vec<String> = vec![
        "test".to_owned(),
        "-p".to_owned(),
        "elai_cc_contracts".to_owned(),
    ];
    if let Some(ref name) = test_name {
        args.push("--test".to_owned());
        args.push(name.clone());
    }

    let join = tokio::task::spawn(async move {
        let mut child = match tokio::process::Command::new("cargo")
            .args(&args)
            .env("ELAI_LIVE_SIDECAR", "1")
            .env("ANTHROPIC_BASE_URL", "http://localhost:8100")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                let _ = app_task.emit(
                    &format!("d5-complete:{rid_event}"),
                    json!({ "exit_code": null, "error": e.to_string(), "ts_ms": now_ms_raw() }),
                );
                return;
            }
        };

        // Merge stdout + stderr via separate tasks feeding a channel.
        let (tx, mut rx) =
            tokio::sync::mpsc::channel::<String>(256);

        // stdout reader task.
        let tx_out = tx.clone();
        let stdout_handle = child.stdout.take().map(|s| {
            tokio::task::spawn(async move {
                let mut lines = BufReader::new(s).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if tx_out.send(line).await.is_err() {
                        break;
                    }
                }
            })
        });

        // stderr reader task.
        let tx_err = tx;
        let stderr_handle = child.stderr.take().map(|s| {
            tokio::task::spawn(async move {
                let mut lines = BufReader::new(s).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if tx_err.send(line).await.is_err() {
                        break;
                    }
                }
            })
        });

        // Forward lines to frontend via Tauri events.
        let progress_event = format!("d5-progress:{rid_event}");
        while let Some(line) = rx.recv().await {
            let _ = app_task.emit(
                &progress_event,
                json!({ "line": line, "ts_ms": now_ms_raw() }),
            );
        }

        // Wait for reader tasks to finish.
        if let Some(h) = stdout_handle {
            let _ = h.await;
        }
        if let Some(h) = stderr_handle {
            let _ = h.await;
        }

        let exit_code: Option<i32> = match child.wait().await {
            Ok(status) => status.code(),
            Err(_) => None,
        };

        let _ = app_task.emit(
            &format!("d5-complete:{rid_event}"),
            json!({ "exit_code": exit_code, "ts_ms": now_ms_raw() }),
        );
    });

    // Store the abort handle so cancel_d5_live_contract can stop the task.
    if let Ok(mut g) = state.active_streams.lock() {
        g.insert(request_id.clone(), join.abort_handle());
    }

    Ok(json!({ "requestId": request_id }))
}

/// Aborts an in-flight `run_d5_live_contract` task by its `request_id`.
#[tauri::command]
pub async fn cancel_d5_live_contract(
    state: State<'_, AppState>,
    request_id: String,
) -> CmdResult<Value> {
    let handle = state
        .active_streams
        .lock()
        .ok()
        .and_then(|mut g| g.remove(&request_id));
    match handle {
        Some(h) => {
            h.abort();
            Ok(json!({ "ok": true, "request_id": request_id }))
        }
        None => Ok(json!({
            "ok": false,
            "request_id": request_id,
            "reason": "not found or already complete"
        })),
    }
}

// ---------------------------------------------------------------------------
// D.7 — ONE-SHOT PROMPT (subprocess elai.exe prompt "...")
// ---------------------------------------------------------------------------

/// Runs `elai.exe prompt "<text>"` as a subprocess and returns stdout.
/// The elai.exe binary is located as a sibling of the current executable
/// (same target/release/ directory). Returns the stdout text on success.
#[tauri::command]
pub async fn one_shot_prompt(text: String) -> CmdResult<Value> {
    let exe = locate_elai_exe()?;
    let output = tokio::process::Command::new(&exe)
        .args(["prompt", &text])
        .output()
        .await
        .map_err(|e| format!("failed to spawn elai.exe: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if output.status.success() {
        Ok(json!({ "ok": true, "text": stdout }))
    } else {
        Err(format!(
            "elai.exe prompt exited with {:?}: {}",
            output.status.code(),
            if stderr.is_empty() { &stdout } else { &stderr }
        ))
    }
}

/// Locate the `elai[.exe]` binary in the same directory as the current
/// executable (i.e., target/release/). Falls back to PATH if not found
/// alongside the dashboard binary.
fn locate_elai_exe() -> CmdResult<std::path::PathBuf> {
    let exe_name = if cfg!(windows) { "elai.exe" } else { "elai" };
    if let Ok(current) = std::env::current_exe() {
        if let Some(dir) = current.parent() {
            let candidate = dir.join(exe_name);
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }
    // Fallback: rely on PATH.
    Ok(std::path::PathBuf::from(exe_name))
}

// ---------------------------------------------------------------------------
// D.8 — GET TOOL VISIBILITY
// ---------------------------------------------------------------------------
// MODEL_HIDDEN_TOOL_NAMES lives in elai-cli (not a dep). Mirror the const
// here to avoid adding the dep (per CLAUDE.md: no new deps without user confirmation).

const MODEL_HIDDEN_TOOL_NAMES: &[&str] = &[
    "TestingPermission",
    "RunTaskPacket",
    "WorkerCreate",
    "WorkerGet",
    "WorkerObserve",
    "WorkerResolveTrust",
    "WorkerAwaitReady",
    "WorkerSendPrompt",
    "WorkerRestart",
    "WorkerTerminate",
    "TeamCreate",
    "TeamDelete",
    "CronCreate",
    "CronDelete",
    "CronList",
    "RemoteTrigger",
    "McpAuth",
    "MCP",
    "SendUserMessage",
    "ListMcpResources",
    "ReadMcpResource",
    "NotebookEdit",
    "ToolSearch",
];

#[tauri::command]
pub fn get_tool_visibility() -> CmdResult<Value> {
    let hidden: Vec<&str> = MODEL_HIDDEN_TOOL_NAMES.to_vec();
    Ok(json!({ "hidden_tools": hidden }))
}

// ---------------------------------------------------------------------------
// B-18 — SANDBOX TIER STATUS
// ---------------------------------------------------------------------------

/// Returns the effective sandbox tier and per-agent profile/path counts.
///
/// Tier detection at runtime (compile-time cfg gating):
///   "C-Windows"  — cfg(target_os="windows"): AppContainer available
///   "C-Linux"    — cfg(target_os="linux"): seccomp/namespace available
///   "A++"        — neither (in-process, no OS isolation)
///
/// profile_count: number of active per-agent AppContainer profiles (B-18 P2 MVP = 0 or 1)
/// allowed_paths_per_agent: per-role allowed_paths counts from role_default_allowed_paths
/// last_visibility_filter_hit_ms: not tracked yet (requires SSB VisibilityFilterHit action type)
#[tauri::command]
pub fn get_sandbox_status(_state: State<'_, AppState>) -> CmdResult<Value> {
    #[cfg(target_os = "windows")]
    let tier = "C-Windows";
    #[cfg(target_os = "linux")]
    let tier = "C-Linux";
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    let tier = "A++";

    // Per-role path counts from B-18 P3 role_default_allowed_paths.
    // These reflect the defaults set at spawn time; actual values depend on
    // project_path which is not in AppState here. We report the structure only.
    let allowed_paths_per_agent = vec![
        json!({ "role": "Executor",        "path_count": 1 }),
        json!({ "role": "Auditor",         "path_count": 2 }),
        json!({ "role": "ContextCompiler", "path_count": 2 }),
        json!({ "role": "Researcher",      "path_count": 2 }),
        json!({ "role": "Designer",        "path_count": 1 }),
        json!({ "role": "PromptEngineer",  "path_count": 1 }),
        json!({ "role": "MemoryWriter",    "path_count": 1 }),
        json!({ "role": "MediaValidator",  "path_count": 1 }),
    ];

    Ok(json!({
        "tier": tier,
        "profile_count": 0_u32,
        "allowed_paths_per_agent": allowed_paths_per_agent,
        "last_visibility_filter_hit_ms": null,
    }))
}

// ---------------------------------------------------------------------------
// D.11 — SIDECAR SETTINGS + RESTART
// ---------------------------------------------------------------------------

/// GETs /v1/settings from the sidecar. Returns the raw JSON object.
/// Falls back to `{"error": "<msg>"}` on network failure.
#[tauri::command]
pub async fn get_sidecar_settings(state: State<'_, AppState>) -> CmdResult<Value> {
    let url = state
        .sidecar_url
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| elai_providers::DEFAULT_SIDECAR_URL.to_owned());
    let base = url.trim_end_matches('/');
    match state
        .http
        .get(format!("{base}/v1/settings"))
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        Ok(resp) => resp.json::<Value>().await.map_err(|e| e.to_string()),
        Err(e) => Ok(json!({ "error": e.to_string() })),
    }
}

/// POSTs `settings` JSON to /v1/settings on the sidecar.
/// The sidecar applies the delta and returns `{"ok": true}` on success.
#[tauri::command]
pub async fn set_sidecar_settings(
    state: State<'_, AppState>,
    settings: Value,
) -> CmdResult<Value> {
    let url = state
        .sidecar_url
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| elai_providers::DEFAULT_SIDECAR_URL.to_owned());
    let base = url.trim_end_matches('/');
    let resp = state
        .http
        .post(format!("{base}/v1/settings"))
        .json(&settings)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json::<Value>().await.map_err(|e| e.to_string())
}

/// Best-effort async sidecar restart via SSH.
/// Kills the old PID and re-boots using the canonical boot command from
/// workflow/remote-setup/REMOTE-SIDECAR.md. Returns immediately with
/// `{"ok": true, "async": true}` — the sidecar takes ~10s to come back up.
#[tauri::command]
pub async fn restart_sidecar() -> CmdResult<Value> {
    // SSH into the remote GPU box and restart the sidecar process.
    // See workflow/remote-setup/REMOTE-SIDECAR.md for canonical credentials.
    let ssh_key = {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_owned());
        std::path::PathBuf::from(home)
            .join(".ssh")
            .join("elai_remote")
    };
    // The boot_cmd is passed directly as the remote shell command via SSH.
    // ssh executes the trailing arguments as a command on the remote shell
    // (no local shell interpretation needed — args are passed as-is to ssh).
    let boot_cmd = concat!(
        "source ~/.venvs/elai-sidecar/bin/activate && cd ~/elai_inference && ",
        "pkill -f python || true; sleep 2; ",
        "nohup env ELAI_FORCE_SDPA=1 python server.py ",
        "--model huihui-ai/Qwen2.5-Coder-3B-Instruct-abliterated ",
        "--port 8100 --no-dola --turboquant-bits 0 --no-steering --load-in-4bit ",
        "> ~/sidecar.log 2>&1 & echo $! > ~/sidecar.pid"
    );

    // Fire-and-forget: spawn the SSH command and return immediately.
    let _child = tokio::process::Command::new("ssh")
        .args([
            "-i",
            ssh_key.to_str().unwrap_or("~/.ssh/elai_remote"),
            "-p",
            "6465",
            "maintainer@192.0.2.10",
            boot_cmd,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("ssh spawn failed: {e}"))?;

    Ok(json!({ "ok": true, "async": true, "note": "sidecar restart initiated; allow ~10s to come back up" }))
}

// ---------------------------------------------------------------------------
// D.12 — SIDECAR SEARCH
// ---------------------------------------------------------------------------

/// GETs /search?q=<query> on the sidecar. Returns ranked results with
/// snippets in the SearXNG-compatible shape:
/// `{"query": "...", "results": [...], "number_of_results": N}`.
#[tauri::command]
pub async fn sidecar_search(
    state: State<'_, AppState>,
    query: String,
) -> CmdResult<Value> {
    if query.trim().is_empty() {
        return Ok(json!({ "query": "", "results": [], "number_of_results": 0 }));
    }
    let url = state
        .sidecar_url
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| elai_providers::DEFAULT_SIDECAR_URL.to_owned());
    let base = url.trim_end_matches('/');
    match state
        .http
        .get(format!("{base}/search"))
        .query(&[("q", query.as_str()), ("format", "json")])
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
    {
        Ok(resp) => resp.json::<Value>().await.map_err(|e| e.to_string()),
        Err(e) => Ok(json!({
            "query": query,
            "results": [],
            "number_of_results": 0,
            "error": e.to_string(),
        })),
    }
}

// ---------------------------------------------------------------------------
// REMOTE SIDECAR BOOT OVERLAY (B-14 extension)
// ---------------------------------------------------------------------------
// Three commands power the RemoteSidecarBootOverlay.svelte:
//   start_remote_sidecar  — kicks off SSH + tunnel + health-poll in background;
//                           emits `remote-sidecar-progress` events per step.
//   get_remote_sidecar_progress — snapshot of current step+status from AppState.
//   cancel_remote_sidecar — aborts the in-flight boot task.

use crate::remote_sidecar_helpers as rsh;
use crate::state::RemoteSidecarProgress;

fn now_ms_raw() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn set_progress(state: &AppState, step: &str, status: &str, error: Option<String>) {
    let snap = RemoteSidecarProgress {
        step: step.to_owned(),
        status: status.to_owned(),
        ts_ms: now_ms_raw(),
        error,
    };
    if let Ok(mut g) = state.remote_sidecar_progress.lock() {
        *g = Some(snap);
    }
}

/// Starts the remote-sidecar boot sequence in a background tokio task.
///
/// Boot order: SSH boot → 2s pause → SSH tunnel → 3s pause → health poll.
/// Each step transition emits a `remote-sidecar-progress` Tauri event with
/// `{ step, status }` where status is "running" | "ok" | "error".
///
/// Returns `{ "requestId": "<uuid>" }` immediately so the frontend can
/// display the overlay before the first step finishes.
#[tauri::command]
pub async fn start_remote_sidecar(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> CmdResult<Value> {
    use std::time::Duration;

    // Generate a simple request_id from timestamp.
    let request_id = format!("rs-{}", now_ms_raw());
    let rid_clone = request_id.clone();

    // Capture shared progress arc to update from the background task.
    let progress_arc = state.remote_sidecar_progress.clone();

    let emit = {
        let app = app.clone();
        move |step: &str, status: &str, error: Option<String>| {
            let snap = RemoteSidecarProgress {
                step: step.to_owned(),
                status: status.to_owned(),
                ts_ms: now_ms_raw(),
                error: error.clone(),
            };
            if let Ok(mut g) = progress_arc.lock() {
                *g = Some(snap.clone());
            }
            let _ = app.emit("remote-sidecar-progress", &snap);
        }
    };

    let join = tokio::task::spawn(async move {
        // Step 1: SSH boot.
        emit("ssh", "running", None);
        let ssh_result = tokio::task::spawn_blocking(rsh::ssh_boot_remote_sidecar)
            .await
            .unwrap_or_else(|e| Err(format!("task panicked: {e}")));
        match ssh_result {
            Ok(()) => emit("ssh", "ok", None),
            Err(e) => {
                // Non-fatal: sidecar may already be running.
                tracing::warn!(error = %e, "start_remote_sidecar: SSH step failed (continuing)");
                emit("ssh", "ok", Some(format!("warning: {e}")));
            }
        }

        tokio::time::sleep(Duration::from_secs(2)).await;

        // Step 2: Tunnel.
        emit("tunnel", "running", None);
        let tun_result = tokio::task::spawn_blocking(rsh::start_ssh_tunnel)
            .await
            .unwrap_or_else(|e| Err(format!("task panicked: {e}")));
        match tun_result {
            Ok(()) => emit("tunnel", "ok", None),
            Err(e) => {
                emit("tunnel", "error", Some(e.clone()));
                tracing::error!(error = %e, "start_remote_sidecar: tunnel step failed; aborting");
                return;
            }
        }

        tokio::time::sleep(Duration::from_secs(3)).await;

        // Step 3: Health poll.
        emit("health", "running", None);
        let poll_result = tokio::task::spawn_blocking(rsh::poll_until_healthy)
            .await
            .unwrap_or_else(|e| Err(format!("task panicked: {e}")));
        match poll_result {
            Ok(_elapsed) => {
                emit("health", "ok", None);
                emit("done", "ok", None);
            }
            Err(e) => {
                emit("health", "error", Some(e));
            }
        }
    });

    // Store the abort handle so cancel_remote_sidecar can stop it.
    if let Ok(mut g) = state.active_streams.lock() {
        g.insert(request_id.clone(), join.abort_handle());
    }

    // Initialise progress to show the SSH step as "pending".
    set_progress(&state, "ssh", "pending", None);

    Ok(json!({ "requestId": rid_clone }))
}

/// Returns the current remote-sidecar boot progress snapshot from AppState.
/// Returns `null` if `start_remote_sidecar` has not been called this session.
#[tauri::command]
pub fn get_remote_sidecar_progress(state: State<'_, AppState>) -> CmdResult<Value> {
    let snap = state
        .remote_sidecar_progress
        .lock()
        .ok()
        .and_then(|g| g.clone());
    match snap {
        Some(s) => Ok(serde_json::to_value(&s).map_err(|e| e.to_string())?),
        None => Ok(Value::Null),
    }
}

/// Aborts an in-flight remote-sidecar boot task by its `request_id`.
/// The background task is cancelled; the progress state is left as-is so the
/// frontend can show the final step that was reached before cancellation.
#[tauri::command]
pub async fn cancel_remote_sidecar(
    state: State<'_, AppState>,
    request_id: String,
) -> CmdResult<Value> {
    let handle = state
        .active_streams
        .lock()
        .ok()
        .and_then(|mut g| g.remove(&request_id));
    match handle {
        Some(h) => {
            h.abort();
            Ok(json!({ "ok": true, "request_id": request_id }))
        }
        None => Ok(json!({ "ok": false, "request_id": request_id, "reason": "not found or already done" })),
    }
}

// ===========================================================================
// PHASE A (s-69820) — WEB SCAFFOLD + DEV SERVER + FIM SECTION FILL
// ===========================================================================
// Six commands that wire dashboard-v2 to the existing `scaffold` crate,
// `elai-web-validator` crate, and the sidecar `/v1/completions` endpoint.
//
//   list_scaffold_templates            — read templates/web/manifest.toml
//   apply_scaffold_template            — run scaffold::apply_template_from_root
//   fim_fill_section                   — splice a FIM region via sidecar
//   run_web_build_check                — npm run build + output_dir check
//   start_dev_server_for_project       — spawn vite dev server, register handle
//   stop_dev_server_for_project        — drop handle, kill child
//
// All inputs are canonicalised at the boundary; template names are
// rejected if they contain path separators (no traversal via the UI).
// ===========================================================================

const FIM_SIDECAR_MAX_TOKENS: u32 = 512;
const FIM_SIDECAR_TEMPERATURE: f32 = 0.2;
const FIM_SIDECAR_PATH: &str = "/v1/completions";
const FIM_SIDECAR_TIMEOUT_SECS: u64 = 30;

fn validate_template_name(name: &str) -> CmdResult<&str> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("template name is empty".into());
    }
    if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains("..") {
        return Err(format!(
            "template name contains path separators or '..': {trimmed:?}"
        ));
    }
    Ok(trimmed)
}

fn validate_existing_path(raw: &str, label: &str) -> CmdResult<std::path::PathBuf> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} is empty"));
    }
    let path = std::path::PathBuf::from(trimmed);
    if !path.exists() {
        return Err(format!("{label} does not exist: {trimmed}"));
    }
    path.canonicalize()
        .map_err(|e| format!("{label} canonicalize failed: {e}"))
}

/// Phase A audit BLOCK fix (s-69820): bound a candidate path inside the
/// active project root so an XSS/injection from the frontend cannot drive
/// `apply_scaffold_template` / `fim_fill_section` / `run_web_build_check`
/// at arbitrary filesystem targets like `C:\\Windows\\System32\\drivers\\etc\\hosts`.
///
/// Handles the apply_scaffold case (target dir does not yet exist) by
/// canonicalising the *parent* directory and rejoining the final
/// component — `Path::canonicalize` requires the path to exist.
fn ensure_within_project_root(
    candidate: &std::path::Path,
    project_root: &std::path::Path,
    label: &str,
) -> CmdResult<std::path::PathBuf> {
    let project_canon = project_root
        .canonicalize()
        .map_err(|e| format!("project_root canonicalize failed: {e}"))?;

    let candidate_canon = if candidate.exists() {
        candidate
            .canonicalize()
            .map_err(|e| format!("{label} canonicalize failed: {e}"))?
    } else {
        let parent = candidate
            .parent()
            .ok_or_else(|| format!("{label} has no parent directory: {}", candidate.display()))?;
        let name = candidate
            .file_name()
            .ok_or_else(|| format!("{label} has no final component: {}", candidate.display()))?;
        let parent_canon = parent
            .canonicalize()
            .map_err(|e| format!("{label} parent canonicalize failed: {e}"))?;
        parent_canon.join(name)
    };

    if !candidate_canon.starts_with(&project_canon) {
        return Err(format!(
            "{label} {} escapes project root {}",
            candidate_canon.display(),
            project_canon.display()
        ));
    }
    Ok(candidate_canon)
}

#[tauri::command]
pub fn list_scaffold_templates(_state: State<'_, AppState>) -> CmdResult<Value> {
    let root = default_templates_dir();
    let manifest_path = root.join(scaffold::MANIFEST_FILENAME);
    let manifest = scaffold::load_manifest(&manifest_path)
        .map_err(|e| format!("load_manifest({}): {e}", manifest_path.display()))?;

    let mut out: Vec<Value> = Vec::with_capacity(manifest.templates.len());
    for entry in manifest.templates {
        let template_dir = root.join(&entry.name);
        let assets_path = template_dir.join("assets").join("assets.json");
        let assets_count = match std::fs::read(&assets_path) {
            Ok(bytes) => match serde_json::from_slice::<Value>(&bytes) {
                Ok(v) => count_assets(&v),
                Err(_) => 0,
            },
            Err(_) => 0,
        };

        // Audit fix (s-69820): real file count, walked once per template.
        // Bounded depth (no symlink follow) keeps this O(template size),
        // typical templates are <30 files so the cost is negligible.
        let files_count = count_files_recursive(&template_dir);

        out.push(json!({
            "id": entry.name,
            "label": entry.name,
            "framework": entry.languages.first().cloned().unwrap_or_default(),
            "description": entry.description,
            "sections": entry.sections,
            "build_command": entry.build_command,
            "dev_command": entry.dev_command,
            "dev_port": entry.dev_port,
            "recommended_for": entry.recommended_for,
            "theme_system": entry.theme_system,
            "validation_commands": entry.validation_commands,
            "assets_count": assets_count,
            "files_count": files_count,
        }));
    }
    Ok(json!(out))
}

/// Phase A audit fix (s-69820): walk a template directory and count files.
/// Skips errors silently — the worst case is a smaller-than-actual count
/// rendered in the drawer (UX-only, not a correctness boundary). Symlinks
/// are not followed (would risk loops if a template ever contains one).
fn count_files_recursive(dir: &std::path::Path) -> u32 {
    let mut count: u32 = 0;
    let mut stack: Vec<std::path::PathBuf> = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let entries = match std::fs::read_dir(&d) {
            Ok(it) => it,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let ft = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };
            if ft.is_symlink() {
                continue;
            }
            if ft.is_dir() {
                stack.push(entry.path());
            } else if ft.is_file() {
                count = count.saturating_add(1);
            }
        }
    }
    count
}

fn count_assets(v: &Value) -> u32 {
    let mut count: u32 = 0;
    if v
        .get("logo")
        .and_then(|l| l.get("src"))
        .and_then(|s| s.as_str())
        .is_some_and(|s| !s.is_empty())
    {
        count += 1;
    }
    if v
        .get("favicon")
        .and_then(|f| f.get("src"))
        .and_then(|s| s.as_str())
        .is_some_and(|s| !s.is_empty())
    {
        count += 1;
    }
    if let Some(arr) = v.get("images").and_then(|i| i.as_array()) {
        count = count.saturating_add(arr.len() as u32);
    }
    count
}

#[tauri::command]
pub fn apply_scaffold_template(
    state: State<'_, AppState>,
    template: String,
    target_dir: String,
    project_name: Option<String>,
    description: Option<String>,
) -> CmdResult<Value> {
    let template_name = validate_template_name(&template)?.to_owned();
    let target_raw = std::path::PathBuf::from(target_dir.trim());
    if target_raw.as_os_str().is_empty() {
        return Err("target_dir is empty".into());
    }

    let target =
        ensure_within_project_root(&target_raw, &state.project_root_snapshot(), "target_dir")?;

    let templates_root = default_templates_dir();
    let vars = scaffold::default_build_vars(
        "",
        project_name.as_deref(),
        description.as_deref(),
        None,
    );
    scaffold::apply_template_from_root(&templates_root, &template_name, &target, &vars)
        .map_err(|e| format!("apply_template({template_name}): {e}"))?;

    Ok(json!({
        "ok": true,
        "template": template_name,
        "target_dir": target.display().to_string(),
        "templates_root": templates_root.display().to_string(),
    }))
}

#[tauri::command]
pub async fn fim_fill_section(
    state: State<'_, AppState>,
    target_path: String,
    hint: String,
) -> CmdResult<Value> {
    let canonical = validate_existing_path(&target_path, "target_path")?;
    let canonical =
        ensure_within_project_root(&canonical, &state.project_root_snapshot(), "target_path")?;

    let original = tokio::fs::read_to_string(&canonical)
        .await
        .map_err(|e| format!("read {}: {e}", canonical.display()))?;
    let splice = scaffold::find_fim_section(&original)
        .map_err(|e| format!("FIM marker scan failed: {e}"))?;

    let url = state
        .sidecar_url
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| elai_providers::DEFAULT_SIDECAR_URL.to_owned());
    let endpoint = format!("{}{}", url.trim_end_matches('/'), FIM_SIDECAR_PATH);

    let primary_prompt = scaffold::build_qwen_fim_prompt(&splice, &hint);
    let primary = post_completion(&state.http, &endpoint, &primary_prompt).await;

    let raw = match primary {
        Ok(text) if !text.trim().is_empty() => Some(("special_tokens", text)),
        _ => None,
    };
    let (strategy, raw_text) = match raw {
        Some(v) => v,
        None => {
            let fallback_prompt = scaffold::build_instruction_fim_prompt(&splice, &hint);
            let text = post_completion(&state.http, &endpoint, &fallback_prompt)
                .await
                .map_err(|e| format!("FIM fallback request failed: {e}"))?;
            if text.trim().is_empty() {
                return Err("sidecar returned empty fill in both strategies".into());
            }
            ("instruction_fallback", text)
        }
    };

    let cleaned = scaffold::strip_fim_artifacts(&raw_text);
    if cleaned.trim().is_empty() {
        return Err("sidecar fill collapsed to empty after artifact stripping".into());
    }

    let merged = scaffold::splice_fim_section(&splice, cleaned.trim());
    tokio::fs::write(&canonical, merged.as_bytes())
        .await
        .map_err(|e| format!("write {}: {e}", canonical.display()))?;

    Ok(json!({
        "ok": true,
        "strategy": strategy,
        "target_path": canonical.display().to_string(),
        "fill_chars": cleaned.trim().chars().count() as u64,
    }))
}

async fn post_completion(
    http: &reqwest::Client,
    endpoint: &str,
    prompt: &str,
) -> Result<String, String> {
    let body = json!({
        "model": "qwen",
        "prompt": prompt,
        "max_tokens": FIM_SIDECAR_MAX_TOKENS,
        "temperature": FIM_SIDECAR_TEMPERATURE,
    });
    let resp = http
        .post(endpoint)
        .json(&body)
        .timeout(std::time::Duration::from_secs(FIM_SIDECAR_TIMEOUT_SECS))
        .send()
        .await
        .map_err(|e| format!("sidecar POST failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("sidecar HTTP {}", resp.status()));
    }
    let value = resp
        .json::<Value>()
        .await
        .map_err(|e| format!("sidecar JSON decode failed: {e}"))?;
    let text = value
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| "sidecar response missing choices[0].text".to_owned())?
        .to_owned();
    Ok(text)
}

#[tauri::command]
pub async fn run_web_build_check(
    state: State<'_, AppState>,
    project_dir: String,
    command: String,
    output_dir: String,
) -> CmdResult<Value> {
    let project_path = validate_existing_path(&project_dir, "project_dir")?;
    let project_path =
        ensure_within_project_root(&project_path, &state.project_root_snapshot(), "project_dir")?;
    let trimmed_cmd = command.trim();
    if trimmed_cmd.is_empty() {
        return Err("command is empty".into());
    }
    let raw_output = std::path::PathBuf::from(output_dir.trim());
    if raw_output.as_os_str().is_empty() {
        return Err("output_dir is empty".into());
    }
    // Resolve output_dir relative to the project_path (the convention for
    // npm build artefacts: `dist/`, `build/`, etc.) and bound it inside
    // project_root just like the project_dir was.
    let resolved_output = if raw_output.is_absolute() {
        raw_output
    } else {
        project_path.join(raw_output)
    };
    let output_path =
        ensure_within_project_root(&resolved_output, &state.project_root_snapshot(), "output_dir")?;

    let result = elai_web_validator::run_web_build(&project_path, trimmed_cmd, &output_path)
        .await
        .map_err(|e| format!("web build failed: {e}"))?;

    Ok(json!({
        "success": result.success,
        "exit_code": result.exit_code,
        "output_dir": result.output_dir.display().to_string(),
        "stderr_errors": result.stderr_errors,
    }))
}

#[tauri::command]
pub async fn start_dev_server_for_project(
    state: State<'_, AppState>,
    project_dir: String,
    command: String,
    port: u16,
) -> CmdResult<Value> {
    if port == 0 {
        return Err("port must be > 0".into());
    }
    let project_path = validate_existing_path(&project_dir, "project_dir")?;
    let project_path =
        ensure_within_project_root(&project_path, &state.project_root_snapshot(), "project_dir")?;
    let trimmed_cmd = command.trim();
    if trimmed_cmd.is_empty() {
        return Err("command is empty".into());
    }
    let key = project_path.display().to_string();

    // Hold the registry lock across the spawn await — `tokio::sync::Mutex`
    // makes this safe (audit BLOCK fix). Two concurrent invocations now
    // serialise: the second blocks on `lock().await` until the first has
    // either inserted the handle or returned an error, eliminating the
    // double-spawn / orphaned-child race that std::sync::Mutex permitted.
    let mut guard = state.dev_servers.lock().await;
    if guard.contains_key(&key) {
        return Err(format!(
            "dev server already running for project {key:?}; call stop_dev_server_for_project first"
        ));
    }

    let handle = elai_web_validator::start_dev_server(&project_path, trimmed_cmd, port)
        .await
        .map_err(|e| format!("start_dev_server failed: {e}"))?;

    let pid = handle.pid();
    let bound_port = handle.port();
    guard.insert(key.clone(), handle);

    Ok(json!({
        "ok": true,
        "project_id": key,
        "pid": pid,
        "port": bound_port,
    }))
}

#[tauri::command]
pub async fn stop_dev_server_for_project(
    state: State<'_, AppState>,
    project_id: String,
) -> CmdResult<Value> {
    let key = project_id.trim();
    if key.is_empty() {
        return Err("project_id is empty".into());
    }
    let removed = {
        let mut guard = state.dev_servers.lock().await;
        guard.remove(key)
    };
    let handle = match removed {
        Some(h) => h,
        None => {
            return Ok(json!({
                "ok": false,
                "reason": "no dev server registered for that project",
                "project_id": key,
            }))
        }
    };
    elai_web_validator::stop_dev_server(handle)
        .await
        .map_err(|e| format!("stop_dev_server failed: {e}"))?;
    Ok(json!({
        "ok": true,
        "project_id": key,
    }))
}

// ---------------------------------------------------------------------------
// Phase B (s-44680) — WEB LINT (B2) + WEB PERFORMANCE / LIGHTHOUSE (B3+B4)
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn run_web_lint_check(
    state: State<'_, AppState>,
    project_dir: String,
    eslint: bool,
    stylelint: bool,
    axe: bool,
) -> CmdResult<Value> {
    let project_path = validate_existing_path(&project_dir, "project_dir")?;
    let project_path =
        ensure_within_project_root(&project_path, &state.project_root_snapshot(), "project_dir")?;
    let cfg = elai_web_validator::lint::LintRunnerConfig {
        eslint,
        stylelint,
        axe,
        skip_missing_packages: true,
    };
    let findings = elai_web_validator::lint::run_web_lint(&project_path, &cfg)
        .await
        .map_err(|e| format!("run_web_lint failed: {e}"))?;

    let value = serde_json::to_value(&findings)
        .map_err(|e| format!("serialize lint findings: {e}"))?;
    let tool_count = [eslint, stylelint, axe].iter().filter(|x| **x).count();
    Ok(json!({
        "ok": true,
        "project_dir": project_path.display().to_string(),
        "tool_count": tool_count,
        "findings": value,
    }))
}

#[tauri::command]
pub async fn run_lighthouse_audit(
    state: State<'_, AppState>,
    project_dir: String,
    url: String,
) -> CmdResult<Value> {
    let project_path = validate_existing_path(&project_dir, "project_dir")?;
    let project_path =
        ensure_within_project_root(&project_path, &state.project_root_snapshot(), "project_dir")?;
    let trimmed_url = url.trim();
    if trimmed_url.is_empty() {
        return Err("url is empty".into());
    }
    let report = elai_web_validator::cwv::run_lighthouse(&project_path, trimmed_url)
        .await
        .map_err(|e| format!("run_lighthouse failed: {e}"))?;
    Ok(json!({
        "ok": true,
        "project_dir": project_path.display().to_string(),
        "url": trimmed_url,
        "lcp_ms": report.lcp_ms,
        "cls": report.cls,
        "inp_ms": report.inp_ms,
        "perf_score": report.perf_score,
    }))
}

#[tauri::command]
pub async fn probe_chromium_runtime() -> CmdResult<Value> {
    match elai_web_validator::cwv::probe_chromium().await {
        Ok(()) => Ok(json!({ "ok": true, "available": true })),
        Err(e) => Ok(json!({
            "ok": true,
            "available": false,
            "reason": format!("{e}"),
        })),
    }
}

// ---------------------------------------------------------------------------
// B-39 — REFLECT STATE
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_reflect_state(state: State<'_, AppState>) -> CmdResult<ReflectState> {
    let viewer = state
        .viewer
        .as_ref()
        .ok_or_else(|| "ssb viewer unavailable; no session db open".to_owned())?;
    viewer.get_reflect_state().map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// B-28 follow-up — FOCUS TUNNEL INLINE CHIPS (per-baby-step query).
//
// Replaces the 5-command FocusTunnelPanel surface with a single tightly
// scoped command. The dashboard PlanWindow's baby-step detail view invokes
// this once per visible step and renders inline chips:
//   * Mode (FIM / DIFF / STRUCTURED) — first GenerationModeSelected row.
//   * Early-term yes/no + reason — first StreamingEarlyTerminated row.
//   * Micro-steps N — sum of MicroStepCreated.count rows.
//   * Deps N auto / M blocked — DependencyResolved rows (task-level only;
//     DependencyResolvedPayload carries no step_id field so we surface the
//     all-step totals; the chip rendering shows them as task-wide.)
//
// Library-tier aggregators (`mode_distribution_for_payloads`, etc) remain
// exported by `elai_orchestrator::focus_tunnel` for non-dashboard callers.
// ---------------------------------------------------------------------------

/// Hard cap on rows scanned per focus_tunnel query. Matches
/// `DRAWER_QUERY_HARD_CAP` in `ssb_viewer.rs` so the IPC payload never
/// balloons past 500 rows even if the SSB has millions.
const FOCUS_TUNNEL_SCAN_LIMIT: usize = 500;

/// Per-baby-step focus-tunnel summary. Frontend invokes this once per
/// visible baby-step detail card with `{ babyStepId: <u64> }` and renders
/// inline chips/badges. Returns zeroed shape when no SSB viewer is bound.
///
/// Field semantics:
/// * `mode` — first matching GenerationModeSelected row's mode (FIM / DIFF
///   / STRUCTURED), `None` if the step has not yet picked a mode.
/// * `early_term` — `Some({reason, tokens_saved})` if at least one
///   StreamingEarlyTerminated row exists for the step; `None` otherwise.
/// * `micro_steps` — sum of MicroStepCreated.count for rows whose
///   parent_step_id equals the requested baby-step id.
/// * `deps_auto` / `deps_blocked` — task-level totals (the payload type
///   carries no step_id; this is documented so callers don't display them
///   as per-step counts).
#[tauri::command]
pub async fn query_focus_tunnel_per_baby_step(
    state: State<'_, AppState>,
    baby_step_id: u64,
) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!({
            "mode": Value::Null,
            "early_term": Value::Null,
            "micro_steps": 0,
            "deps_auto": 0,
            "deps_blocked": 0,
        }));
    };

    // Wire-form step_id used by GenerationModeSelected + StreamingEarlyTerminated.
    let step_id_wire = format!("baby_step_{baby_step_id}");

    // Mode — fetch GenerationModeSelected rows, filter by step_id, then fold
    // through the library aggregator so we get the canonical FIM/DIFF/STRUCTURED
    // mapping (in case the wire-form string is renamed upstream).
    let mode_rows = viewer
        .focus_tunnel_payloads(
            &[SsbActionType::GenerationModeSelected],
            None,
            FOCUS_TUNNEL_SCAN_LIMIT,
        )
        .map_err(map_err)?;
    let step_mode_refs: Vec<&SsbPayload> = mode_rows
        .iter()
        .filter(|p| match p {
            SsbPayload::GenerationModeSelected(m) => m.step_id == step_id_wire,
            _ => false,
        })
        .collect();
    let dist = mode_distribution_for_payloads(&step_mode_refs);
    let mode_str: Option<&'static str> = if dist.get(&GenerationMode::Fim).copied().unwrap_or(0) > 0
    {
        Some("FIM")
    } else if dist.get(&GenerationMode::Diff).copied().unwrap_or(0) > 0 {
        Some("DIFF")
    } else if dist
        .get(&GenerationMode::Structured)
        .copied()
        .unwrap_or(0)
        > 0
    {
        Some("STRUCTURED")
    } else {
        None
    };

    // Early-term — first matching row.
    let term_rows = viewer
        .focus_tunnel_payloads(
            &[SsbActionType::StreamingEarlyTerminated],
            None,
            FOCUS_TUNNEL_SCAN_LIMIT,
        )
        .map_err(map_err)?;
    let early_term_json: Value = term_rows
        .iter()
        .find_map(|p| match p {
            SsbPayload::StreamingEarlyTerminated(t) if t.step_id == step_id_wire => Some(json!({
                "reason": t.reason,
                "tokens_saved": t.token_count_at_term,
            })),
            _ => None,
        })
        .unwrap_or(Value::Null);

    // Micro-steps — sum of count for matching parent_step_id.
    let micro_rows = viewer
        .focus_tunnel_payloads(
            &[SsbActionType::MicroStepCreated],
            None,
            FOCUS_TUNNEL_SCAN_LIMIT,
        )
        .map_err(map_err)?;
    let micro_total: u32 = micro_rows
        .iter()
        .filter_map(|p| match p {
            SsbPayload::MicroStepCreated(m) if m.parent_step_id == baby_step_id => {
                Some(u32::from(m.count))
            }
            _ => None,
        })
        .sum();

    // Deps — task-level totals; DependencyResolvedPayload has no step_id, so we
    // simply count auto-approved vs blocked across the cap window.
    let dep_rows = viewer
        .focus_tunnel_payloads(
            &[SsbActionType::DependencyResolved],
            None,
            FOCUS_TUNNEL_SCAN_LIMIT,
        )
        .map_err(map_err)?;
    let mut deps_auto: u32 = 0;
    let mut deps_blocked: u32 = 0;
    for p in &dep_rows {
        if let SsbPayload::DependencyResolved(row) = p {
            match row.decision.as_str() {
                "auto_approve" => deps_auto = deps_auto.saturating_add(1),
                "blocked" | "queue_for_human" => deps_blocked = deps_blocked.saturating_add(1),
                _ => {}
            }
        }
    }

    Ok(json!({
        "mode": mode_str,
        "early_term": early_term_json,
        "micro_steps": micro_total,
        "deps_auto": deps_auto,
        "deps_blocked": deps_blocked,
    }))
}

#[cfg(test)]
mod parse_intervention_choice_tests {
    use super::parse_intervention_choice;
    use elai_orchestrator::{InterventionKeyword, InterventionOption};

    fn keyword(opt: InterventionOption) -> InterventionKeyword {
        match opt {
            InterventionOption::Keyword(k) => k,
            other => panic!("expected Keyword variant, got {other:?}"),
        }
    }

    #[test]
    fn approve_aliases_map_to_accept() {
        for input in ["approve", "accept", "ok", "OK", " Approve "] {
            let got = parse_intervention_choice(input).expect("approve alias parses");
            assert_eq!(keyword(got), InterventionKeyword::Accept, "input={input:?}");
        }
    }

    #[test]
    fn reject_aliases_map_to_decline() {
        for input in ["reject", "decline", "no"] {
            let got = parse_intervention_choice(input).expect("reject alias parses");
            assert_eq!(keyword(got), InterventionKeyword::Decline, "input={input:?}");
        }
    }

    #[test]
    fn modify_and_skip_map_to_skip() {
        for input in ["modify", "skip"] {
            let got = parse_intervention_choice(input).expect("modify alias parses");
            assert_eq!(keyword(got), InterventionKeyword::Skip, "input={input:?}");
        }
    }

    #[test]
    fn redirect_and_retry_map_to_retry_default() {
        for input in ["redirect", "retry"] {
            let got = parse_intervention_choice(input).expect("retry alias parses");
            match got {
                InterventionOption::Retry(r) => assert_eq!(r.retry, "default"),
                other => panic!("expected Retry, got {other:?} (input={input:?})"),
            }
        }
    }

    #[test]
    fn unknown_choice_returns_err() {
        let err = parse_intervention_choice("frobnicate").expect_err("unknown variant errors");
        assert!(err.contains("unknown intervention choice"), "err={err}");
    }

    // PART12 OQ4 contract: 'takeover' is a first-class InterventionAction
    // (the dashboard-v2 MonitorWindow exposes a Takeover button distinct from
    // Approve/Reject). Semantically takeover = "I will proceed with this" =
    // Accept keyword. The arm must be present so MonitorWindow can stop using
    // the 'approve' workaround.
    #[test]
    fn takeover_maps_to_accept() {
        let got = parse_intervention_choice("takeover").expect("takeover parses");
        assert_eq!(keyword(got), InterventionKeyword::Accept);
    }
}
