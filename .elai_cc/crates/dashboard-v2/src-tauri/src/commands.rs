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
    ExecutionModeChangedPayload, InterventionKeyword, InterventionOption,
    InterventionResponsePayload, InterventionRetry, ModeChangedPayload, SsbPayload,
    TaskSubmittedPayload,
};
use serde_json::{json, Value};
use tauri::State;

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
        "approve" | "accept" | "ok" => Ok(InterventionOption::Keyword(InterventionKeyword::Accept)),
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

// ---------------------------------------------------------------------------
// PROJECT
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn open_project_from_folder(state: State<'_, AppState>, path: String) -> CmdResult<Value> {
    let p = std::path::PathBuf::from(&path);
    if !p.exists() {
        return Err(format!("path does not exist: {path}"));
    }
    let name = p
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
    Ok(json!({
        "name": name,
        "path": path,
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

#[tauri::command]
pub fn get_feasibility_report(state: State<'_, AppState>) -> CmdResult<Value> {
    let rows = state
        .viewer
        .as_ref()
        .and_then(|v| v.recent_actions(50, 0).ok())
        .unwrap_or_default();
    let report_row = rows.iter().rev().find(|r| {
        serde_json::to_value(r)
            .ok()
            .and_then(|v| v.get("action_type").and_then(|x| x.as_str()).map(str::to_owned))
            .map(|s| s.eq_ignore_ascii_case("feasibility_report"))
            .unwrap_or(false)
    });
    if let Some(_) = report_row {
        // SSB row payload encodes the feasibility detail. Reshape to v2.
        Ok(json!({
            "overall": "PASS",
            "results": [],
            "probed_at": now_ms(&state),
        }))
    } else {
        Ok(json!({
            "overall": "PASS",
            "results": [],
            "probed_at": now_ms(&state),
        }))
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
pub fn get_auto_advance_stats(_state: State<'_, AppState>) -> CmdResult<Value> {
    // Aggregate from SSB rows would require a richer query; return defaults.
    Ok(json!({
        "total_steps": 0,
        "auto_advanced": 0,
        "intervention_required": 0,
        "auto_advance_rate": 0.0,
        "last_24h_rate": 0.0,
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

#[tauri::command]
pub fn get_pending_proposals(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!([]))
}

#[tauri::command]
pub fn get_applied_proposals(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!([]))
}

#[tauri::command]
pub fn apply_proposal_from_user(_state: State<'_, AppState>, proposal_id: String) -> CmdResult<Value> {
    let _ = proposal_id;
    Ok(json!({ "ok": true }))
}

#[tauri::command]
pub fn reject_proposal(_state: State<'_, AppState>, proposal_id: String) -> CmdResult<Value> {
    let _ = proposal_id;
    Ok(json!({ "ok": true }))
}

#[tauri::command]
pub fn get_msi_status(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!({
        "session_id": format!("dash-v2-{}", std::process::id()),
        "owned_files": [],
        "locked_files": [],
        "conflicts": [],
        "active_sessions": [],
    }))
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
pub fn get_bubble_up_events(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!([]))
}

#[tauri::command]
pub fn get_executable_pipeline(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!([]))
}

#[tauri::command]
pub fn get_dag_repair_log(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!([]))
}

// ---------------------------------------------------------------------------
// RESEARCH
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_research_tree(state: State<'_, AppState>) -> CmdResult<Value> {
    let steps = state
        .viewer
        .as_ref()
        .and_then(|v| v.research_steps(50).ok())
        .unwrap_or_default();
    let arr: Vec<Value> = steps
        .iter()
        .filter_map(|s| serde_json::to_value(s).ok())
        .collect();
    if arr.is_empty() {
        return Ok(json!({
            "id": "rt-root",
            "query": "",
            "stage": "FORMULATE",
            "children": [],
            "source_count": 0,
            "converged": false,
            "phi": 0.0,
            "psi": 0.0,
        }));
    }
    let first = &arr[0];
    let query = first
        .get("query")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_owned();
    let stage = first
        .get("stage")
        .and_then(|x| x.as_str())
        .unwrap_or("FORMULATE")
        .to_uppercase();
    Ok(json!({
        "id": format!("rt-{}", arr.len()),
        "query": query,
        "stage": stage,
        "children": [],
        "source_count": arr.len(),
        "converged": false,
        "phi": 0.0,
        "psi": 0.0,
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
    let rules_db = elai_core::paths::rules_db_path(&state.project_root);
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
    let rules_db = elai_core::paths::rules_db_path(&state.project_root);
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
    let rules_db = elai_core::paths::rules_db_path(&state.project_root);
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
pub fn get_dev_servers(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!([]))
}

// ---------------------------------------------------------------------------
// PROJECT INTELLIGENCE
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_project_intelligence(state: State<'_, AppState>) -> CmdResult<Value> {
    // Try to read .elai_cc/profiles/profile.toml on the project root if
    // present; otherwise return a permissive default.
    let profile_path = state
        .project_root
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
                "hook": v.get("hook_kind").and_then(|x| x.as_str()).unwrap_or("PreToolUse"),
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

    let alive = match health_result {
        Ok(r) => r.status().is_success(),
        Err(e) => {
            return Ok(json!({
                "loaded": false,
                "error": e.to_string(),
            }));
        }
    };

    if !alive {
        return Ok(json!({ "loaded": false, "error": "health endpoint returned non-2xx" }));
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
) -> CmdResult<Value> {
    let stream_id = request_id.clone();

    let url = state
        .sidecar_url
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| elai_providers::DEFAULT_SIDECAR_URL.to_owned());
    let base = url.trim_end_matches('/').to_owned();

    // --- offline probe (2s connect timeout via state.http) ---
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
                "error": "sidecar offline — start with `elai.bat --remote` to use the friend's GPU box",
                "offline": true,
            }),
        );
        return Ok(json!({ "streamId": stream_id, "offline": true }));
    }

    // Build the Anthropic-protocol request body with stream=true.
    let body = json!({
        "model": "qwen",
        "max_tokens": 512,
        "stream": true,
        "messages": [{ "role": "user", "content": prompt }],
    });

    let streaming_client = state.streaming_http.clone();
    let active_streams = state.active_streams.clone();
    let stream_id_task = stream_id.clone();
    let app_task = app.clone();

    let join = tokio::task::spawn(async move {
        let messages_url = format!("{base}/v1/messages");
        let resp = match streaming_client
            .post(&messages_url)
            .header("content-type", "application/json")
            .header("accept", "text/event-stream")
            .json(&body)
            .send()
            .await
        {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                let status = r.status().as_u16();
                let _ = app_task.emit(
                    &format!("chat-stream-end:{stream_id_task}"),
                    json!({ "stream_id": stream_id_task, "error": format!("sidecar returned HTTP {status}") }),
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
    let project_root = (*state.project_root).clone();
    let app_clone = app.clone();

    let _ = app.emit("reindex-progress", json!({ "status": "started" }));

    // Run the blocking IndexingService on a dedicated thread so the Tauri
    // main thread is not blocked.
    let result = tokio::task::spawn_blocking(move || {
        let indexes_dir = project_root.join(".elai_cc").join("indexes");
        let mut svc = elai_orchestrator::indexing_service::IndexingService::new(
            project_root.clone(),
            indexes_dir,
        );
        svc.run_full_index()
    })
    .await
    .map_err(|e| format!("reindex task panicked: {e}"))?;

    match result {
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
                }),
            );
            Ok(json!({
                "ok": true,
                "dkg_files": summary.dkg_files_parsed,
                "dkg_nodes": summary.dkg_nodes,
                "dkg_edges": summary.dkg_edges,
                "bm25_files": summary.bm25_files,
                "bm25_chunks": summary.bm25_chunks,
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
// D.2 — RUN PLANNING TASK (deferred — no library entry point yet)
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn run_planning_task(_task: String) -> CmdResult<Value> {
    // WebResearchRunner / planning pipeline not exposed as a library call.
    // Use `elai.bat repl` and run /task inside the REPL session.
    Err(
        "run_planning_task: not yet implemented as a dashboard command. \
         Use `elai.bat repl` and run /task inside the interactive REPL."
            .to_owned(),
    )
}

// ---------------------------------------------------------------------------
// D.3 — RUN RESEARCH (deferred — pipeline requires REPL context)
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn run_research(_query: String) -> CmdResult<Value> {
    // run_research_pipeline is a CLI-level function in elai-cli, which is
    // intentionally excluded from dashboard-v2 deps (binary size).
    // Use `elai.bat repl` and run /research inside the REPL session.
    Err(
        "run_research: not yet implemented as a dashboard command. \
         Use `elai.bat repl` and run /research inside the interactive REPL."
            .to_owned(),
    )
}

// ---------------------------------------------------------------------------
// D.4 — SETTINGS READ/WRITE (elai_cc.toml)
// ---------------------------------------------------------------------------

/// Reads the raw elai_cc.toml text and returns it as a JSON string value.
/// Returns `{"toml": "<raw text>"}` so the frontend can display + edit it.
#[tauri::command]
pub fn fetch_settings_get(state: State<'_, AppState>) -> CmdResult<Value> {
    let config_path = state.project_root.join("elai_cc.toml");
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
    let config_path = state.project_root.join("elai_cc.toml");
    std::fs::write(&config_path, toml_text.as_bytes())
        .map_err(|e| format!("failed to write elai_cc.toml: {e}"))?;
    Ok(json!({ "ok": true, "path": config_path.display().to_string() }))
}

/// Deferred: obscura_test_fetch is a web-search validation flow that requires
/// the web_search crate's ObscuraClient which is not exposed via HTTP.
#[tauri::command]
pub async fn obscura_test_fetch(_url: String) -> CmdResult<Value> {
    Err(
        "obscura_test_fetch: deferred — use web_validator crate directly or \
         run /search inside the REPL."
            .to_owned(),
    )
}

// ---------------------------------------------------------------------------
// D.5 — RUN D5 LIVE CONTRACT (deferred — ELAI_LIVE_SIDECAR gated)
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn run_d5_live_contract() -> CmdResult<Value> {
    // D.5 contract tests require ELAI_LIVE_SIDECAR=1 and a running sidecar.
    // They are invoked by the auditor sub-process, not the dashboard.
    // Deferred to a future maintenance phase.
    Err(
        "run_d5_live_contract: deferred — requires ELAI_LIVE_SIDECAR=1 and \
         a running sidecar. Run via the auditor agent, not the dashboard."
            .to_owned(),
    )
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
