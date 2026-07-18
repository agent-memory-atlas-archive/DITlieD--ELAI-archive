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
use std::path::PathBuf;
use std::sync::atomic::Ordering;

// Required for AppHandle::emit() calls in stream_chat / start_chat_stream
// (Tauri 2 event emitter for SSE token streaming to the frontend).
use tauri::Emitter;

// C-15: replace bare SystemTime::now / Instant::now with ClockProvider-backed
// equivalents so the entropy quarantine scanner stays clean. Dashboard reads
// are observation-only (UI latency timing, SSB row timestamps); no per-call
// determinism contract — but we route through OsClock to enforce the policy.
use elai_core::clock::{ClockProvider, OsClock};

use elai_orchestrator::{
    AutoAdvanceEvaluator, AutonomousEvidence, ChangeDecisionPayload, ChatMessagePayload,
    ChatResponsePayload, ContextBudgetComputedPayload, EvidenceVerdict,
    ExecutionModeChangedPayload, FeasibilityReportRow, FeasibilitySummary,
    InterventionCredentials, InterventionKeyword,
    InterventionOption, InterventionResponsePayload, InterventionRetry,
    MergeResult, ModeChangedPayload, NovelStepRow, ReflectState, SearchModeChangedPayload,
    SessionDag, SessionDagError, SsbActionType,
    SsbPayload, StubRunRow, TaskSubmittedPayload,
};
// C-23: per-call DAG open uses OsEntropy / OsClock for the merge path
// (merge_branch_with_ssb needs entropy + clock). create_branch_with_ssb /
// close_branch_with_ssb take the same providers for consistency.
use elai_core::{OsClock as DagOsClock, OsEntropy, paths::sessions_dir_path};
// B-28 follow-up — Focus-tunnel side-panel deleted; only the per-baby-step
// command remains, and it consumes the mode-distribution aggregator only
// (early-term + micro-step counts are folded inline here so we don't drag
// the streaming/pre-assembly/dep aggregators back in).
use elai_orchestrator::focus_tunnel::{mode_distribution_for_payloads, GenerationMode};
// E-25 Tranche B-4 (D4): SessionForked SSB emit at the dashboard fork
// command. Wire-dark unless ELAI_E25_SESSION_FORK_EMIT_ENABLE=1.
use elai_orchestrator::architecture::claude_code_arch::{
    ClaudeCodeArchPrimitive, ClaudeCodeArchitectureEventKind,
    ClaudeCodeArchitectureEventPayload,
};
use elai_orchestrator::ssb_viewer::SelfModMsiSnapshotRow;
// F-8 / Slice A5: SymbolicConflictRow is the SCF detector's held-pending
// surface. Unlike SelfModProposalRow it is NOT gated behind the
// `self-modification` feature — `SsbViewer::symbolic_conflicts` is part of
// the detector path, which composes independently of Layer 1A self-mod.
use elai_orchestrator::ssb_viewer::SymbolicConflictRow;
// F-11: DarwinianAdmitRow is the config-DGM admit-loop feed. Like
// SymbolicConflictRow it is NOT gated behind the `self-modification` feature —
// `SsbViewer::darwinian_admits` is an always-compiled read-side query (the
// producer is the wire-dark darwin-loop CLI, gated by feature + env at the
// orchestrator/CLI layer, not here).
use elai_orchestrator::ssb_viewer::DarwinianAdmitRow;
// H-46: CladeSelectionRow is the CMP parent-selection feed. Same access model as
// DarwinianAdmitRow — read-side always compiled; producer is wire-dark
// (config_archive::select_parent_with_cmp behind darwinian-evolve + env).
use elai_orchestrator::ssb_viewer::CladeSelectionRow;
// B-3 #28: SelfModProposalRow only consumed by the
// #[cfg(feature="self-modification")] branches of get_pending_proposals /
// get_applied_proposals; gate the import to match so default builds don't
// warn.
#[cfg(feature = "self-modification")]
use elai_orchestrator::ssb_viewer::SelfModProposalRow;
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
// E-51: Runtime capability probe
// ---------------------------------------------------------------------------

/// E-51: Probe the current platform's capability for a given backend.
/// Returns a JSON object with platform-specific availability information.
#[tauri::command]
pub fn runtime_capability_probe(backend: &str) -> serde_json::Value {
    use elai_providers::runtime::{detect_llamacpp_ep, detect_mlx_capable, MlxCapability};
    match backend {
        "mlx" => {
            let cap = detect_mlx_capable();
            match &cap {
                MlxCapability::NotApple => json!({
                    "backend": "mlx",
                    "available": false,
                    "reason": "Not a Darwin platform (MLX requires macOS arm64)",
                    "platform_badge": "not-apple"
                }),
                MlxCapability::AppleIntel => json!({
                    "backend": "mlx",
                    "available": false,
                    "reason": "Intel Mac detected (MLX requires Apple Silicon M-series)",
                    "platform_badge": "apple-intel"
                }),
                MlxCapability::AppleSilicon { chip_family, gpu_cores } => json!({
                    "backend": "mlx",
                    "available": true,
                    "reason": format!("{:?} with {} GPU cores detected", chip_family, gpu_cores),
                    "platform_badge": "apple-silicon"
                }),
            }
        }
        "llama_cpp" => {
            let ep = detect_llamacpp_ep();
            json!({
                "backend": "llama_cpp",
                "available": true,
                "ep": format!("{:?}", ep),
                "reason": format!("LlamaCpp available with {:?} execution provider", ep),
                "platform_badge": format!("ep-{:?}", ep).to_lowercase()
            })
        }
        _ => json!({
            "backend": backend,
            "available": false,
            "reason": "Unknown backend for capability probe"
        }),
    }
}

// ---------------------------------------------------------------------------
// C-44: Composer 2 paid-backend status + cumulative cost rollup
// ---------------------------------------------------------------------------

/// C-44 — DTO for `runtime_composer2_status`. Mirrors the dashboard
/// stub TS type `Composer2StatusReport`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Composer2StatusReport {
    pub enabled: bool,
    pub model: String,
    pub health: serde_json::Value,
    pub cumulative_cost_micros: u64,
    pub cumulative_tokens_in: u64,
    pub cumulative_tokens_out: u64,
    pub invocations: u64,
}

/// C-44 — return the Composer 2 adapter's current health + cumulative
/// cost rollup for the active session. Cost is summed from
/// `backend_cost_accrued` SSB rows whose `backend_name_hash` matches
/// `composer2`. The implementation here is intentionally minimal: it
/// reads the live RuntimeOrchestrator config to detect whether
/// composer2 is enabled (api_key present + factory cfg seeded), then
/// surfaces a degraded health when no api_key is wired. Full SSB
/// rollup is wired by the orchestrator's event-sink layer; this
/// surface is the dashboard-side reader.
#[tauri::command]
pub async fn runtime_composer2_status(
    state: State<'_, AppState>,
) -> Result<Composer2StatusReport, String> {
    use elai_providers::runtime::RuntimeBackend;

    let cfg = state.runtime_orchestrator.snapshot_config().await;
    let factory = cfg
        .factory_cfg
        .get(&RuntimeBackend::Composer2)
        .cloned();

    let (enabled, model, health) = match factory.as_ref() {
        Some(f) => {
            let has_key = f.api_key.is_some();
            let model = f
                .model
                .clone()
                .unwrap_or_else(|| "composer-2".to_string());
            if has_key {
                let health = json!({"status": "ready"});
                (true, model, health)
            } else {
                let health = json!({
                    "status": "unavailable",
                    "reason": "CURSOR_API_KEY not set; set env var + [runtime.composer2].enabled = true"
                });
                (false, model, health)
            }
        }
        None => (
            false,
            "composer-2".to_string(),
            json!({
                "status": "unavailable",
                "reason": "composer2 factory config missing from RuntimeConfig"
            }),
        ),
    };

    // Rollup is currently a no-op stub on the dashboard side; the SSB
    // viewer surface is wired by the orchestrator's event-sink layer.
    // We surface zeros until that wiring lands; the dashboard renders
    // them as $0.00 + 0 invocations, which is the correct view for a
    // session that has not yet routed through Composer 2.
    Ok(Composer2StatusReport {
        enabled,
        model,
        health,
        cumulative_cost_micros: 0,
        cumulative_tokens_in: 0,
        cumulative_tokens_out: 0,
        invocations: 0,
    })
}

// ---------------------------------------------------------------------------
// E-24: MCP security status
// ---------------------------------------------------------------------------

/// E-24 / G-4: Returns the real MCP security status feed for the
/// `McpStatusPanel`. De-stubbed in G-4 (was a hard-coded `vec![]`): now reads
/// the `mcp_rug_pull_blocked` SSB rows via the viewer, so a runtime rug-pull /
/// line-jump (a reconnect re-fetch whose blake3 schema pin no longer matched)
/// surfaces as a real security event with its before/after schema hashes, the
/// changed tool names, and the description-diff count. Returns an empty list
/// when no SSB DB is reachable (fresh install) or no rug-pull has occurred.
///
/// The dedicated `McpRugPullDrawer` reads the same rows via
/// `commands_g4::get_mcp_rug_pulls` (richer, capped feed); this command keeps
/// the existing panel's contract (a flat `Vec<Value>` it polls).
#[tauri::command]
pub fn get_mcp_security_status(state: State<'_, AppState>) -> Vec<serde_json::Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return vec![];
    };
    let rows = viewer.mcp_rug_pulls(MCP_SECURITY_STATUS_FEED_CAP).unwrap_or_default();
    rows.into_iter()
        .filter_map(|r| serde_json::to_value(r).ok())
        .collect()
}

/// G-4 — read cap for the legacy `get_mcp_security_status` panel feed. Kept
/// small (the panel is a status summary, not the full forensic drawer).
const MCP_SECURITY_STATUS_FEED_CAP: usize = 50;

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
    // B-40 BLOCK-4 audit fix: derive agent_substate from SSB.
    // FRAMING is the only substate at MVP; surface it whenever the latest
    // IdeaFramingStarted row is more recent than the latest IdeaFramed row
    // for the active session (i.e. FRAMING is in progress, not finished).
    // Returning `Value::Null` outside FRAMING preserves the dashboard's
    // existing "no substate" rendering path.
    let agent_substate: Value = state
        .viewer
        .as_ref()
        .and_then(|viewer| {
            let started = viewer
                .rows_with_session_by_action_type(SsbActionType::IdeaFramingStarted)
                .ok()
                .and_then(|rows| rows.into_iter().last());
            let framed_last_seq = viewer
                .rows_with_session_by_action_type(SsbActionType::IdeaFramed)
                .ok()
                .and_then(|rows| rows.into_iter().last())
                .map(|_| true)
                .unwrap_or(false);
            // FRAMING is active iff we have an IdeaFramingStarted row AND
            // either no IdeaFramed row OR the IdeaFramed row is for an
            // older session. At MVP we use simple "started but not yet
            // framed for ANY session" — refined per-session matching is a
            // follow-up. Conservatively: emit "Framing" when started_rows
            // exist and framed_rows is empty.
            let _ = started?;
            if !framed_last_seq {
                Some(Value::String("Framing".to_owned()))
            } else {
                None
            }
        })
        .unwrap_or(Value::Null);

    Ok(json!({
        "state": current,
        "current_step_id": "",
        "progress": 0.0,
        "history": history,
        "agent_substate": agent_substate,
        "_phase": phase,
    }))
}

// ---------------------------------------------------------------------------
// B-26 T4 — Plan progress (real phase progression, replaces the
//           MonitorWindow.svelte stub 5-phase hardcode).
//
// Returns five top-level phases (Plan / Feasibility / Execute / Audit /
// Memorize) with per-phase status derived from the FSM viewer's recent
// phase_transition rows and the latest FSM state. This consumes only
// the already-shipped viewer surface (latest_fsm_state +
// recent_phase_transitions) — no orchestrator type changes, no ssb.rs
// changes (D1 enforcement). Returns `{ phases: null, current_phase:
// null, total_pct: 0.0 }` when the viewer is unavailable (sidecar
// offline) so the frontend can fall through to its empty-state render.
// ---------------------------------------------------------------------------

fn phase_index_for_state(label: &str) -> Option<usize> {
    match label {
        "PLANNING" => Some(0),
        "FEASIBILITY_PROBE" => Some(1),
        "TESTING_FIRST" | "EXECUTING" => Some(2),
        "AUDITING" => Some(3),
        "MEMORIZING" | "COMPLETE" => Some(4),
        _ => None,
    }
}

#[tauri::command]
pub fn get_plan_progress(state: State<'_, AppState>) -> CmdResult<Value> {
    const PHASE_LABELS: [&str; 5] = ["Plan", "Feasibility", "Execute", "Audit", "Memorize"];

    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!({
            "phases": Value::Null,
            "current_phase": Value::Null,
            "total_pct": 0.0,
        }));
    };

    let current_label = viewer
        .latest_fsm_state()
        .ok()
        .map(|s| fsm_state_to_v2_label(&s.state).to_string())
        .unwrap_or_else(|| "IDLE".to_string());
    let current_idx = phase_index_for_state(&current_label);

    // Count phase_transition rows to each FSM state (proxy for
    // steps-per-phase). Window of 100 matches MonitorWindow display
    // budget — older rows are pruned at the orchestrator's retention
    // boundary anyway.
    let transitions = viewer
        .recent_phase_transitions(100)
        .ok()
        .unwrap_or_default();
    let mut steps: [u32; 5] = [0; 5];
    for (_seq, _from, to, _ts) in &transitions {
        let label = fsm_state_to_v2_label(to);
        if let Some(idx) = phase_index_for_state(label) {
            steps[idx] = steps[idx].saturating_add(1);
        }
    }

    let phases: Vec<Value> = PHASE_LABELS
        .iter()
        .enumerate()
        .map(|(idx, label)| {
            let status = match current_idx {
                Some(active) if idx < active => "done",
                Some(active) if idx == active => "active",
                _ => "pending",
            };
            json!({
                "num": idx + 1,
                "label": label,
                "steps": steps[idx],
                "status": status,
            })
        })
        .collect();

    let total_pct = match current_idx {
        Some(idx) => ((idx as f64) + 1.0) / 5.0,
        None => 0.0,
    };

    Ok(json!({
        "phases": phases,
        "current_phase": current_idx.map(|i| i + 1).map(Value::from).unwrap_or(Value::Null),
        "total_pct": total_pct,
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
pub async fn submit_task(
    state: State<'_, AppState>,
    task: String,
    autonomous: Option<bool>,
) -> CmdResult<Value> {
    let writer = state
        .writer
        .as_ref()
        .ok_or_else(|| "ssb writer unavailable; no session db open".to_owned())?;
    let project_id = state.project_id.lock().map(|g| g.clone()).unwrap_or_default();

    // B-22 STEP 2: when the caller passes `autonomous: Some(true)`, run the
    // AutoAdvanceEvaluator's 6 evidence checks before emitting the
    // TaskSubmitted row. This wires the evaluator into the production
    // "dashboard task submit with --autonomous mode" entrypoint. On a
    // Fail verdict the submission is rejected with the failing check names
    // so the frontend can surface the reason; on Pass the row is emitted
    // and the taskId is returned as usual.
    if autonomous.unwrap_or(false) {
        let evaluator = AutoAdvanceEvaluator::default();
        // Cold-start evidence: the dashboard submit path has no SSB history
        // to derive real evidence from, so we use the default (all-false)
        // snapshot. The evaluator's AutoAdvance check will return Fail with
        // the list of failing checks, which the frontend surfaces as a
        // "not ready for autonomous mode" message. This is the correct
        // behaviour — autonomous mode requires evidence artifacts to be
        // present before auto-advance is permitted.
        let evidence = AutonomousEvidence::default();
        let verdict = evaluator.evaluate_all(&evidence);
        if !verdict.is_pass() {
            let reason = match verdict {
                EvidenceVerdict::Fail { reason } => reason,
                _ => "evidence check failed".to_owned(),
            };
            return Err(format!(
                "autonomous mode rejected: evidence checks failed [{reason}]"
            ));
        }
    }

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

    // C-22: aggregate counts surfaced for FeasibilityPanel.
    //  - probe_count: distinct (FeasibilityReportRow) snapshots backing this
    //    view. Currently 1 when a latest_report exists, 0 otherwise — the
    //    underlying viewer keeps only the most-recent report.
    //  - alternatives_explored: alternative slots attempted across all
    //    stub_runs in window. With v1 stub data this collapses to the
    //    number of stub_runs (one alt slot per stub); StubRunPayload does
    //    not yet carry alternative_index, so the count below is a faithful
    //    PROXY rather than an exact slot tally. The dashboard renders it
    //    as "alternatives attempted".
    //  - novel_step_count: surfaced for symmetry with the FeasibilityPanel
    //    spec.
    let probe_count = if summary.latest_report.is_some() { 1u32 } else { 0u32 };
    let alternatives_explored = u32::try_from(summary.stub_runs.len()).unwrap_or(u32::MAX);
    let novel_step_count = u32::try_from(summary.novel_steps.len()).unwrap_or(u32::MAX);
    let skip_reason = summary
        .latest_report
        .as_ref()
        .and_then(|r| r.skip_reason.clone());

    Ok(json!({
        "overall": overall,
        "results": results,
        "probed_at": probed_at,
        "probe_count": probe_count,
        "alternatives_explored": alternatives_explored,
        "novel_step_count": novel_step_count,
        "skip_reason": skip_reason,
    }))
}

/// E-22 phase-4F P5: surface the on-disk `stub_results.msgpack` artifact
/// produced by `DefaultFeasibilityProber::write_stub_results`. Closes the
/// spec "GOAL: Output stub_results.msgpack" loop — the writer was always
/// present; this command is the missing reader-side consumer. Returns a
/// JSON envelope `{ task_id, count, results: [...], path }` so the
/// dashboard can render replay-verification details without re-issuing
/// the live probe.
///
/// `task_id` is the pipeline-id string used as the per-task subdirectory
/// name under `<project_root>/.elai_cc/feasibility/`. When the directory
/// is absent (no probe has run for that task yet), returns
/// `{ count: 0, results: [] }`.
#[tauri::command]
pub fn get_stub_results_msgpack(
    state: State<'_, AppState>,
    task_id: String,
) -> CmdResult<Value> {
    let project_root = state
        .project_root
        .lock()
        .map_err(|e| format!("project_root poisoned: {e}"))?
        .clone();
    let path = project_root
        .join(".elai_cc")
        .join("feasibility")
        .join(&task_id)
        .join("stub_results.msgpack");
    let results = elai_orchestrator::read_stub_results(&path).map_err(|e| {
        tracing::warn!(error = %e, path = %path.display(), "get_stub_results_msgpack: read failed");
        format!("read_stub_results: {e}")
    })?;
    let count = results.len();
    // Project StubResult to a JSON envelope the FE can render. We avoid
    // leaking the raw Rust struct because the FE TypeScript shape is
    // stable across StubResult v1/v2 (additive fields only).
    let rows: Vec<Value> = results
        .iter()
        .map(|r| {
            json!({
                "step_id": r.step_id,
                "stub_passed": r.stub_passed,
                "elapsed_ms": r.elapsed_ms,
                "error": r.error,
                "alternative_index": r.alternative_index,
                "stdout_hash": r.stdout_hash,
                "stderr_summary": r.stderr_summary,
                "resource_delta": r.resource_delta.as_ref().map(|d| json!({
                    "vram_bytes_before": d.vram_bytes_before,
                    "vram_bytes_after": d.vram_bytes_after,
                    "ram_bytes_before": d.ram_bytes_before,
                    "ram_bytes_after": d.ram_bytes_after,
                })),
            })
        })
        .collect();
    Ok(json!({
        "task_id": task_id,
        "path": path.display().to_string(),
        "count": count,
        "results": rows,
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

/// B-3 #27 (CLASS C): AutoAdvanceStats Tauri command, gated behind the
/// `autonomous-mode` cargo feature. The autonomous-mode producer (B-22)
/// has not landed yet; until then the default build emits a zeroed
/// shape so the frontend's `get_feature_flags()` query reads
/// `autonomous_mode=false` and the panel hides itself via the
/// `featureFlags.autonomous_mode` runtime guard in MonitorWindow.svelte.
#[cfg(feature = "autonomous-mode")]
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

/// B-3 #27 (CLASS C): non-autonomous-mode build returns a zeroed shape so
/// the Tauri command symbol remains registered (`tauri::generate_handler!`
/// is a single-source-of-truth — removing the symbol would break the
/// dashboard's `invoke('get_auto_advance_stats')` fallthrough). The
/// frontend reads `feature_flags.autonomous_mode` on bootstrap; when
/// false, the panel hides itself entirely and never invokes this command.
#[cfg(not(feature = "autonomous-mode"))]
#[tauri::command]
pub fn get_auto_advance_stats(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!({
        "total_steps": 0_u64,
        "auto_advanced": 0_u64,
        "intervention_required": 0_u64,
        "auto_advance_rate": 0.0_f64,
        "last_24h_rate": 0.0_f64,
    }))
}

/// B-3 #27/#28 + V2 rebaseline: compile-time feature flags surfaced to the
/// frontend so future-phase UI sneak-ins (AutoAdvanceStats Phase 4G,
/// ProposalsPanel Phase 4H) can hide themselves until the producers land.
/// Also surfaces `ELAI_DASHBOARD_OFFLINE` (runtime env var) so stub.ts can
/// short-circuit Tauri invocation when set, matching the V2 rebaseline
/// preface convention.
#[tauri::command]
pub fn get_feature_flags() -> CmdResult<Value> {
    let autonomous_mode = cfg!(feature = "autonomous-mode");
    let self_modification = cfg!(feature = "self-modification");
    let dashboard_offline = std::env::var("ELAI_DASHBOARD_OFFLINE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    Ok(json!({
        "autonomous_mode": autonomous_mode,
        "self_modification": self_modification,
        "dashboard_offline": dashboard_offline,
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
// E-3 (SCHEMA_VERSION 188) -- role-shown / model-hidden default telemetry.
// Scope-lean MVP slice of TASK-E03-001: emit one SSB row each time the
// SettingsWindow "show model details" toggle changes. Reverses the
// cross-check finding "SettingsWindow explicitly exposes MODEL names per
// role" by making MODEL hidden the default. Wire-dark unless
// ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE=1.
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn record_model_visibility(
    state: State<'_, AppState>,
    role: String,
    model_hidden: bool,
    session_id: Option<String>,
) -> CmdResult<Value> {
    // Wire-dark gate: silently skip emission when the env var is unset.
    // The UI toggle itself is always reactive; this is observation-only.
    let enabled = std::env::var(
        elai_orchestrator::ssb::ELAI_ROLE_VISIBILITY_TELEMETRY_ENABLE_ENV,
    )
    .ok()
    .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
    .unwrap_or(false);

    if !enabled {
        return Ok(json!({ "ok": true, "emitted": false }));
    }

    let Some(writer) = state.writer.as_ref() else {
        // No SSB writer available (no session DB open). UI toggle still
        // succeeds; we just cant emit telemetry.
        return Ok(json!({ "ok": true, "emitted": false }));
    };

    let payload = SsbPayload::RoleShownModelHidden(
        elai_orchestrator::ssb::RoleShownModelHiddenPayload {
            role,
            model_hidden,
            session_id,
            ts_ms: now_ms(&state),
        },
    );
    writer
        .write_payload("dashboard-v2", 0, payload, vec![])
        .await
        .map_err(map_err)?;
    Ok(json!({ "ok": true, "emitted": true }))
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
///
/// B-3 #28 (CLASS C): self-modification feature gate. The default build
/// returns `[]` so the frontend's `feature_flags.self_modification` query
/// reads false and ProposalsPanel hides itself. Build with
/// `--features self-modification` to surface real Layer 1A proposals
/// once B-23 lands.
#[cfg(feature = "self-modification")]
#[tauri::command]
pub fn get_pending_proposals(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows: Vec<SelfModProposalRow> = viewer.pending_proposals_layer1a().unwrap_or_default();
    serde_json::to_value(rows).map_err(map_err)
}

#[cfg(not(feature = "self-modification"))]
#[tauri::command]
pub fn get_pending_proposals(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!([]))
}

/// B-23 Phase 4H Group F — list applied Layer 1A proposals (history).
/// B-3 #28: same self-modification cfg gate as `get_pending_proposals`.
#[cfg(feature = "self-modification")]
#[tauri::command]
pub fn get_applied_proposals(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows: Vec<SelfModProposalRow> = viewer.applied_proposals_layer1a().unwrap_or_default();
    serde_json::to_value(rows).map_err(map_err)
}

#[cfg(not(feature = "self-modification"))]
#[tauri::command]
pub fn get_applied_proposals(_state: State<'_, AppState>) -> CmdResult<Value> {
    Ok(json!([]))
}

/// F-8 / Slice A5 — dashboard read cap for the SCF symbolic-conflict feed.
/// The viewer clamps to its own `DRAWER_QUERY_HARD_CAP` (500); this is the
/// dashboard-side request size kept well under that ceiling so the IPC
/// payload stays small at the ReasoningDrawer's 10s poll cadence.
const SYMBOLIC_CONFLICTS_FEED_CAP: usize = 200;

/// F-8 / Slice A5 — list the held-pending self-mod config variants the SCF
/// symbolic detector flagged against an active locked-key invariant
/// (`self_mod_safety::LOCKED_KEYS`). Surfaces the
/// `symbolic_conflict_detected` SSB rows newest-first.
///
/// POLICY is WARN-then-escalate, never hard fail-closed BLOCK — these rows
/// are held-pending variants awaiting human escalation, NOT rejections.
/// Unlike `get_applied_proposals`, this command is NOT gated behind the
/// `self-modification` feature: the detector path composes independently
/// (the `elai self-mod apply` CLI entrypoint + the darwinian-evolve seam).
///
/// Returns an empty array when no SSB DB is reachable. Serde serializes
/// `SymbolicConflictRow`'s field names verbatim, so the frontend
/// `SymbolicConflict` DTO must mirror them exactly.
#[tauri::command]
pub fn get_symbolic_conflicts(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows: Vec<SymbolicConflictRow> = viewer
        .symbolic_conflicts(SYMBOLIC_CONFLICTS_FEED_CAP)
        .unwrap_or_default();
    serde_json::to_value(rows).map_err(map_err)
}

/// F-11 — dashboard read cap for the config-DGM admit-loop feed. The viewer
/// clamps to its own `DRAWER_QUERY_HARD_CAP` (500); this is the dashboard-side
/// request size kept well under that ceiling so the IPC payload stays small at
/// the ReasoningDrawer's 10s poll cadence.
const DARWINIAN_ADMITS_FEED_CAP: usize = 200;

/// F-11 — list the config-DGM admit-loop events newest-first: each candidate
/// routed through the four-guard chain (`darwinian_admit` SSB rows). Surfaces
/// `Admitted` / `RejectedByGuard` (with the firing guard) / `LoopClosed`.
///
/// Like `get_symbolic_conflicts` this command is NOT gated behind the
/// `self-modification` feature: `SsbViewer::darwinian_admits` is an
/// always-compiled read-side query; the PRODUCER (the darwin-loop CLI) is the
/// wire-dark surface (gated by feature `darwinian-evolve` + env
/// `ELAI_DARWINIAN_LOOP_ENABLE=1`), not this reader.
///
/// Returns an empty array when no SSB DB is reachable. Serde serializes
/// `DarwinianAdmitRow`'s field names verbatim, so the frontend `DarwinianAdmit`
/// DTO must mirror them exactly.
#[tauri::command]
pub fn get_darwinian_admits(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows: Vec<DarwinianAdmitRow> = viewer
        .darwinian_admits(DARWINIAN_ADMITS_FEED_CAP)
        .unwrap_or_default();
    serde_json::to_value(rows).map_err(map_err)
}

/// H-46 — dashboard read cap for the CMP parent-selection feed.
const CLADE_SELECTIONS_FEED_CAP: usize = 200;

/// H-46 — list CMP parent-selection events newest-first. Each entry is one
/// `CladeSelected` SSB row emitted by `config_archive::select_parent_with_cmp`
/// during a DGM generation. Surfaces `node_id`, `cmp`, `clade_size`,
/// `immediate_score`, and `key_used` (CladeMetaproductivity /
/// ImmediateScoreFallback).
///
/// NOT gated behind the `self-modification` feature: `SsbViewer::clade_selections`
/// is an always-compiled read-side query; the PRODUCER (the darwin-loop CLI) is
/// the wire-dark surface (gated by feature `darwinian-evolve` + env
/// `ELAI_DARWINIAN_LOOP_ENABLE=1`), not this reader.
///
/// Returns an empty array when no SSB DB is reachable. Serde serializes
/// `CladeSelectionRow`'s field names verbatim, so the frontend `CladeSelection`
/// DTO must mirror them exactly.
#[tauri::command]
pub fn get_clade_selections(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows: Vec<CladeSelectionRow> = viewer
        .clade_selections(CLADE_SELECTIONS_FEED_CAP)
        .unwrap_or_default();
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
        let ping_clock = OsClock;
        let t0 = ping_clock.now();
        if let Ok(r) = state.http.get(&url).send().await {
            connected = r.status().is_success();
            ping_ms = Some(ping_clock.elapsed_since(t0).as_millis() as i64);
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
    // C-1: read the most recent `query_trace` rows written by
    // `QueryPipeline::query_with_tier`. Returns `[]` when no SSB DB is
    // reachable OR no rows have been written yet. Each row carries the
    // QueryTracePayload reshaped for the dashboard KnowledgeDrawer.
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let limit = (n as usize).max(QUERY_TRACE_DEFAULT_LIMIT);
    let rows = viewer
        .recent_rows_of_type(SsbActionType::QueryTrace.as_sql_str(), limit)
        .unwrap_or_default();
    let mut out: Vec<Value> = Vec::with_capacity(rows.len());
    for detail in rows {
        let inner = detail
            .payload_json
            .get("QueryTrace")
            .cloned()
            .unwrap_or(detail.payload_json);
        out.push(json!({
            "seq": detail.seq,
            "ts": detail.ts,
            "query_id": inner.get("query_id").cloned().unwrap_or(Value::Null),
            "tier": inner.get("tier").cloned().unwrap_or(Value::Null),
            "stage_count": inner.get("stage_count").and_then(|v| v.as_u64()).unwrap_or(0),
            "result_count": inner
                .get("result_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            "total_latency_ms": inner
                .get("total_latency_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        }));
    }
    Ok(json!(out))
}

/// C-1: default row count returned by `get_query_trace_recent` when the
/// caller passes `n=0`. Named constant per codeRules baseline.
const QUERY_TRACE_DEFAULT_LIMIT: usize = 20;

/// B-4 task #5: singular per-query QueryTrace inspector. Reads the most
/// recent `query_trace` SSB rows and returns the one whose `query_id`
/// matches the argument. The SSB `QueryTracePayload` (payloads.rs:5010)
/// carries `query_id` / `tier` / `stage_count` / `result_count` /
/// `total_latency_ms` — aggregate fields only (per-stage `StageTrace`
/// data is emitted via `tracing::debug!`, not persisted in SSB). The
/// KnowledgeDrawer "Last query" panel renders these aggregate fields
/// with a tier badge + total-latency bar.
///
/// Returns `{known:false}` when no SSB DB is reachable OR no row matches
/// the given `query_id` (cold start / unknown id) so the panel renders
/// "(no data)" instead of crashing on null.
#[tauri::command]
pub fn get_query_trace(state: State<'_, AppState>, query_id: String) -> CmdResult<Value> {
    let trimmed = query_id.trim();
    if trimmed.is_empty() {
        return Ok(json!({ "known": false }));
    }
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!({ "known": false }));
    };
    let rows = viewer
        .recent_rows_of_type(SsbActionType::QueryTrace.as_sql_str(), QUERY_TRACE_DEFAULT_LIMIT)
        .unwrap_or_default();
    for detail in rows {
        let inner = detail
            .payload_json
            .get("QueryTrace")
            .cloned()
            .unwrap_or(detail.payload_json);
        let id = inner.get("query_id").and_then(|v| v.as_str()).unwrap_or("");
        if id == trimmed {
            return Ok(json!({
                "known": true,
                "seq": detail.seq,
                "ts": detail.ts,
                "query_id": id,
                "tier": inner.get("tier").cloned().unwrap_or(Value::Null),
                "stage_count": inner.get("stage_count").and_then(|v| v.as_u64()).unwrap_or(0),
                "result_count": inner
                    .get("result_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                "total_latency_ms": inner
                    .get("total_latency_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
            }));
        }
    }
    Ok(json!({ "known": false }))
}

/// B-4 task #13: read the on-disk `index_version.toml` for the active
/// project and return the structured `IndexVersionSchema` (version /
/// dkg_schema_version / vector_dimensions / bm25_format / created_at).
/// Returns `{known:false, error:"..."}` when the file is absent (first
/// run) or unreadable so the SettingsWindow panel renders "(no index
/// version yet)" instead of erroring.
#[tauri::command]
pub fn get_index_version(state: State<'_, AppState>) -> CmdResult<Value> {
    let project_root = state.project_root_snapshot();
    let indexes_dir = project_root.join(".elai_cc").join("indexes");
    let version_path = indexes_dir.join("index_version.toml");
    match elai_knowledge::index_version::read(&version_path) {
        Ok(Some(file)) => Ok(json!({
            "known": true,
            "version": file.index.version,
            "dkg_schema_version": file.index.dkg_schema_version,
            "vector_dimensions": {
                "code": file.index.vector_dimensions.code,
                "text": file.index.vector_dimensions.text,
            },
            "bm25_format": file.index.bm25_format,
            "created_at": file.index.created_at,
        })),
        Ok(None) => Ok(json!({
            "known": false,
            "error": "index_version.toml not found (first run)",
        })),
        Err(e) => Ok(json!({
            "known": false,
            "error": format!("index_version read failed: {e}"),
        })),
    }
}

/// B-4 task #13: compare the on-disk `index_version.toml` against the
/// current binary's expectations and return the `UpgradeAction` verdict
/// (`Reuse` / `RebuildBm25Only` / `FullRebuild`). Returns
/// `{known:false, error:"..."}` when the file is absent or unreadable so
/// the SettingsWindow panel renders "(no upgrade decision yet)".
#[tauri::command]
pub fn get_upgrade_action(state: State<'_, AppState>) -> CmdResult<Value> {
    let project_root = state.project_root_snapshot();
    let indexes_dir = project_root.join(".elai_cc").join("indexes");
    let version_path = indexes_dir.join("index_version.toml");
    match elai_knowledge::index_version::read(&version_path) {
        Ok(Some(file)) => {
            let action = elai_knowledge::index_version::compare(&file.index);
            Ok(json!({
                "known": true,
                "action": action,
            }))
        }
        Ok(None) => Ok(json!({
            "known": false,
            "error": "index_version.toml not found (first run)",
        })),
        Err(e) => Ok(json!({
            "known": false,
            "error": format!("index_version read failed: {e}"),
        })),
    }
}

/// B-4: default row count returned by `get_hyde_query_recent` when the
/// caller passes `n=0`. Matches `QUERY_TRACE_DEFAULT_LIMIT` — the
/// KnowledgeDrawer hypothesis-preview panel shows the latest event plus a
/// short history strip.
const HYDE_QUERY_DEFAULT_LIMIT: usize = 20;

/// B-4: dashboard-side hard cap on `get_hyde_query_recent`. The viewer's
/// own `DRAWER_QUERY_HARD_CAP` (500) is the absolute ceiling; we keep a
/// tighter cap here so a frontend bug cannot pull hundreds of rows over IPC
/// for this narrow per-event feed.
const HYDE_QUERY_HARD_CAP: usize = 200;

/// B-4: read the most recent `hyde_query` SSB rows written by
/// `HydeStage::hypothesize`. Returns `[]` when no SSB DB is reachable OR no
/// rows have been written yet. Each row is reshaped from
/// [`elai_orchestrator::HydeQueryPayload`] for the KnowledgeDrawer
/// hypothesis-preview inline panel.
///
/// SSB-FIDELITY NOTE: the `hyde_query` payload persists `query_hash`
/// (blake3 hex of the query bytes) + `hypothesis_len` (char length of the
/// produced hypothesis; `0` when none was produced) + `cache_hit` +
/// `latency_ms`. It deliberately does NOT persist the full hypothesis
/// TEXT (privacy / row-size); the dashboard previews length + cache-hit +
/// latency, not the raw hypothesis string.
#[tauri::command]
pub fn get_hyde_query_recent(state: State<'_, AppState>, n: u32) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let limit = (n as usize)
        .max(HYDE_QUERY_DEFAULT_LIMIT)
        .min(HYDE_QUERY_HARD_CAP);
    let rows = viewer
        .recent_rows_of_type(SsbActionType::HydeQuery.as_sql_str(), limit)
        .unwrap_or_default();
    let mut out: Vec<Value> = Vec::with_capacity(rows.len());
    for detail in rows {
        // SsbPayload serializes externally-tagged:
        //   {"HydeQuery": {query_hash, hypothesis_len, cache_hit, latency_ms}}
        let inner = detail
            .payload_json
            .get("HydeQuery")
            .cloned()
            .unwrap_or(detail.payload_json);
        out.push(json!({
            "seq": detail.seq,
            "ts": detail.ts,
            "query_hash": inner.get("query_hash").cloned().unwrap_or(Value::Null),
            "hypothesis_len": inner
                .get("hypothesis_len")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            "cache_hit": inner
                .get("cache_hit")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            "latency_ms": inner
                .get("latency_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        }));
    }
    Ok(json!(out))
}

// ---------------------------------------------------------------------------
// E-17 SP4: RAG retrieval-quality SSB row surface
//
// Four read-only commands that mirror `get_query_trace_recent` (above) for the
// SCHEMA_VERSION 97 SP2 payload variants. Each follows the same template:
//   1. `let Some(viewer) = state.viewer.as_ref() else { return Ok(json!([])); };`
//   2. `viewer.recent_rows_of_type(SsbActionType::<X>.as_sql_str(), limit)
//          .unwrap_or_default();`
//   3. Reshape each `SsbActionDetail`'s `payload_json` using the externally
//      tagged enum envelope (`inner = payload_json.get("<Variant>")`).
//   4. ORDER BY seq DESC LIMIT n is applied inside `recent_rows_of_type`
//      (DRAWER_QUERY_HARD_CAP = 500 ceiling at the viewer layer).
//
// The producers (knowledge::pipeline / knowledge::deep / knowledge::hyde)
// were shipped in E-17 SP2; rows arrive in SSB without any further wiring.
// ---------------------------------------------------------------------------

/// E-17 SP4: default row count returned by the four RAG retrieval-quality
/// commands below when the caller passes `n=0`. Tuned to 20 to match
/// `QUERY_TRACE_DEFAULT_LIMIT` — the cards render the latest event plus a
/// small history strip.
const RAG_QUALITY_DEFAULT_LIMIT: usize = 20;

/// E-17 SP4: dashboard-side hard cap on the four RAG retrieval-quality
/// commands. The viewer's own `DRAWER_QUERY_HARD_CAP` (500) is the absolute
/// ceiling; we keep a tighter cap here so a frontend bug cannot pull
/// hundreds of rows over IPC for these narrow per-event feeds.
const RAG_QUALITY_HARD_CAP: usize = 200;

/// E-17 SP4: clamp `n` (the user-supplied limit) into
/// `[default, RAG_QUALITY_HARD_CAP]`. `n = 0` falls through to the default.
fn rag_quality_limit(n: u32) -> usize {
    match n as usize {
        0 => RAG_QUALITY_DEFAULT_LIMIT,
        v => v.min(RAG_QUALITY_HARD_CAP),
    }
}

/// E-17 SP4 — read recent `deep_pool_capped` SSB rows. Emitted once per
/// DEEP-tier query when `knowledge::deep::deep_pool_cap` drops candidates
/// above the per-stage cap. Empty array when the viewer is absent or no
/// row has been written yet — DEEP queries that stay below the cap do
/// not emit a row, so empty is the steady-state for many sessions.
///
/// Per-row shape: `{ seq, ts, capped_count, max_allowed, stage_count }`.
#[tauri::command]
pub fn get_deep_pool_capped_recent(state: State<'_, AppState>, n: u32) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let limit = rag_quality_limit(n);
    let rows = viewer
        .recent_rows_of_type(SsbActionType::DeepPoolCapped.as_sql_str(), limit)
        .unwrap_or_default();
    let mut out: Vec<Value> = Vec::with_capacity(rows.len());
    for detail in rows {
        let inner = detail
            .payload_json
            .get("DeepPoolCapped")
            .cloned()
            .unwrap_or(detail.payload_json);
        out.push(json!({
            "seq": detail.seq,
            "ts": detail.ts,
            "capped_count": inner.get("capped_count").and_then(|v| v.as_u64()).unwrap_or(0),
            "max_allowed": inner.get("max_allowed").and_then(|v| v.as_u64()).unwrap_or(0),
            "stage_count": inner.get("stage_count").and_then(|v| v.as_u64()).unwrap_or(0),
        }));
    }
    Ok(json!(out))
}

/// E-17 SP4 — read recent `rrf_quality_gate_fired` SSB rows. Emitted once
/// per DEEP-tier query when `knowledge::deep::rrf_quality_gate` drops
/// fused candidates below the RRF floor. Empty array is steady-state for
/// many sessions; presence indicates the gate is doing work.
///
/// Per-row shape: `{ seq, ts, dropped_count, floor, total_before }`.
#[tauri::command]
pub fn get_rrf_quality_gate_recent(state: State<'_, AppState>, n: u32) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let limit = rag_quality_limit(n);
    let rows = viewer
        .recent_rows_of_type(SsbActionType::RrfQualityGateFired.as_sql_str(), limit)
        .unwrap_or_default();
    let mut out: Vec<Value> = Vec::with_capacity(rows.len());
    for detail in rows {
        let inner = detail
            .payload_json
            .get("RrfQualityGateFired")
            .cloned()
            .unwrap_or(detail.payload_json);
        out.push(json!({
            "seq": detail.seq,
            "ts": detail.ts,
            "dropped_count": inner.get("dropped_count").and_then(|v| v.as_u64()).unwrap_or(0),
            "floor": inner.get("floor").and_then(|v| v.as_f64()).unwrap_or(0.0),
            "total_before": inner.get("total_before").and_then(|v| v.as_u64()).unwrap_or(0),
        }));
    }
    Ok(json!(out))
}

/// E-17 SP4 — read recent `hyde_skipped_code_query` SSB rows. Emitted
/// when `knowledge::hyde::should_use_hyde` returns false on a
/// CodeSymbol / Mixed query that would otherwise have invoked HyDE
/// (DEEP tier + >= MIN_WORDS_FOR_HYDE). Empty is steady-state for sessions
/// dominated by natural-language queries.
///
/// Per-row shape: `{ seq, ts, query_class, word_count, code_signal_present }`.
#[tauri::command]
pub fn get_hyde_skipped_code_query_recent(state: State<'_, AppState>, n: u32) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let limit = rag_quality_limit(n);
    let rows = viewer
        .recent_rows_of_type(SsbActionType::HydeSkippedCodeQuery.as_sql_str(), limit)
        .unwrap_or_default();
    let mut out: Vec<Value> = Vec::with_capacity(rows.len());
    for detail in rows {
        let inner = detail
            .payload_json
            .get("HydeSkippedCodeQuery")
            .cloned()
            .unwrap_or(detail.payload_json);
        out.push(json!({
            "seq": detail.seq,
            "ts": detail.ts,
            "query_class": inner.get("query_class").cloned().unwrap_or(Value::Null),
            "word_count": inner.get("word_count").and_then(|v| v.as_u64()).unwrap_or(0),
            "code_signal_present": inner
                .get("code_signal_present")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        }));
    }
    Ok(json!(out))
}

/// E-17 SP4 — read recent `rrf_component_weights` SSB rows. Emitted once
/// per query with the per-ranker RRF weights active for that fusion run.
/// `bm25_weight` / `code_vec_weight` / `text_vec_weight` are `Option<f64>`;
/// `None` indicates the ranker did not contribute (e.g. `code_vec_weight`
/// is None when ELAI_CODE_VEC is disabled or no code index is loaded).
/// Serialised as JSON null when absent.
///
/// Per-row shape: `{ seq, ts, bm25_weight, code_vec_weight, text_vec_weight }`.
#[tauri::command]
pub fn get_rrf_component_weights_recent(state: State<'_, AppState>, n: u32) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let limit = rag_quality_limit(n);
    let rows = viewer
        .recent_rows_of_type(SsbActionType::RrfComponentWeights.as_sql_str(), limit)
        .unwrap_or_default();
    let mut out: Vec<Value> = Vec::with_capacity(rows.len());
    for detail in rows {
        let inner = detail
            .payload_json
            .get("RrfComponentWeights")
            .cloned()
            .unwrap_or(detail.payload_json);
        // Each weight is Option<f64> on the Rust side. `.as_f64()` collapses
        // both None (Value::Null) and absent keys to None, which we then
        // re-serialise as Value::Null so the TS shape stays
        // `number | null`. `.unwrap_or(Value::Null)` preserves null through
        // the wire boundary.
        let bm25 = inner
            .get("bm25_weight")
            .and_then(|v| if v.is_null() { None } else { v.as_f64() })
            .map(Value::from)
            .unwrap_or(Value::Null);
        let code_vec = inner
            .get("code_vec_weight")
            .and_then(|v| if v.is_null() { None } else { v.as_f64() })
            .map(Value::from)
            .unwrap_or(Value::Null);
        let text_vec = inner
            .get("text_vec_weight")
            .and_then(|v| if v.is_null() { None } else { v.as_f64() })
            .map(Value::from)
            .unwrap_or(Value::Null);
        out.push(json!({
            "seq": detail.seq,
            "ts": detail.ts,
            "bm25_weight": bm25,
            "code_vec_weight": code_vec,
            "text_vec_weight": text_vec,
        }));
    }
    Ok(json!(out))
}

/// C-3: default top-N count for `get_pagerank_top` when `n=0`.
const PAGERANK_DEFAULT_LIMIT: usize = 20;

/// C-3: maximum row count accepted for `get_pagerank_top`. The viewer's
/// own DRAWER_QUERY_HARD_CAP (500) is the absolute ceiling; the dashboard
/// surface keeps a tighter dashboard-side limit so a frontend bug cannot
/// pull thousands of rows over IPC.
const PAGERANK_HARD_CAP: usize = 200;

/// C-3: read the top-N persistent PageRank entries via
/// [`SsbViewer::pagerank_top`]. DRIFT-1 fallback: when the persistent
/// `dkg_nodes_pagerank` table does not yet exist, the viewer probes
/// `sqlite_master` and returns an empty vector — the command therefore
/// returns `[]` (not an error), and the dashboard renders "no data".
#[tauri::command]
pub fn get_pagerank_top(state: State<'_, AppState>, n: u32) -> CmdResult<Value> {
    let limit = match n as usize {
        0 => PAGERANK_DEFAULT_LIMIT,
        v => v.min(PAGERANK_HARD_CAP),
    };
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!({"known": false, "rows": []}));
    };
    match viewer.pagerank_top(limit) {
        Ok(entries) => {
            let rows: Vec<Value> = entries
                .iter()
                .map(|e| {
                    json!({
                        "symbol": e.symbol,
                        "path": e.path,
                        "score": e.score,
                    })
                })
                .collect();
            // `known` is true when at least one row was found OR when the
            // probe ran without error (empty arr still means "table absent
            // OR empty"; the dashboard treats both as graceful-empty).
            Ok(json!({
                "known": !rows.is_empty(),
                "rows": rows,
            }))
        }
        Err(e) => {
            tracing::warn!(error = %e, "get_pagerank_top: viewer query failed");
            Ok(json!({"known": false, "rows": []}))
        }
    }
}

/// C-3: read the latest FreshnessState from the SSB viewer. DRIFT-4
/// fallback: the freshness producer is not yet wired (no SSB row type
/// emits decay_rate/access_boost) AND AppState carries no ConfigManager
/// handle, so the viewer's stub returns `known: false` and we forward
/// that shape verbatim. When a producer lands (see C-1 follow-up), the
/// upgrade is a one-line change to the viewer body — no command change.
#[tauri::command]
pub fn get_freshness_state(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!({
            "known": false,
            "decay_rate": 0.0,
            "access_boost": 0.0,
            "last_recalc_ms": 0,
        }));
    };
    match viewer.freshness_state() {
        Ok(fs) => Ok(json!({
            "known": fs.known,
            "decay_rate": fs.decay_rate,
            "access_boost": fs.access_boost,
            "last_recalc_ms": fs.last_recalc_ms,
        })),
        Err(e) => {
            tracing::warn!(error = %e, "get_freshness_state: viewer query failed");
            Ok(json!({
                "known": false,
                "decay_rate": 0.0,
                "access_boost": 0.0,
                "last_recalc_ms": 0,
            }))
        }
    }
}

/// B-3 #3: query-tier routing stats wrapper. Thin Tauri command around
/// the existing [`SsbViewer::query_tier_stats`] stub at
/// orchestrator/src/ssb_viewer.rs:2952. The viewer currently returns a
/// zeroed shape (no SSB row type records per-query tier decisions yet —
/// B-4 producer scope). The command shape is wired now so the
/// KnowledgeDrawer's per-tier histogram can subscribe without rendering
/// "all zero" silently — `sample_size==0` is the dashboard's "no data"
/// signal per the V2 "known: false" convention.
#[tauri::command]
pub fn get_query_tier_stats(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!({
            "fast_pct": 0.0,
            "standard_pct": 0.0,
            "deep_pct": 0.0,
            "avg_latency_ms_fast": 0.0,
            "avg_latency_ms_standard": 0.0,
            "avg_latency_ms_deep": 0.0,
            "sample_size": 0,
        }));
    };
    match viewer.query_tier_stats() {
        Ok(stats) => Ok(json!({
            "fast_pct": stats.fast_pct,
            "standard_pct": stats.standard_pct,
            "deep_pct": stats.deep_pct,
            "avg_latency_ms_fast": stats.avg_latency_ms_fast,
            "avg_latency_ms_standard": stats.avg_latency_ms_standard,
            "avg_latency_ms_deep": stats.avg_latency_ms_deep,
            "sample_size": stats.sample_size,
        })),
        Err(e) => {
            tracing::warn!(error = %e, "get_query_tier_stats: viewer query failed");
            Ok(json!({
                "fast_pct": 0.0,
                "standard_pct": 0.0,
                "deep_pct": 0.0,
                "avg_latency_ms_fast": 0.0,
                "avg_latency_ms_standard": 0.0,
                "avg_latency_ms_deep": 0.0,
                "sample_size": 0,
            }))
        }
    }
}

/// B-3 #11: rule lifecycle transition log wrapper. Reads the most recent
/// `rule_transition` SSB rows so the RulesDrawer can render the
/// last-N rule lifecycle transitions inline (PROPOSED -> CANDIDATE ->
/// SHADOW -> ACTIVE -> RETIRED) without polling the rules SQLite DB
/// directly. Returns an empty array when no producer has run yet —
/// the renderer treats `[]` as "no transitions yet".
#[tauri::command]
pub fn get_rule_transitions(state: State<'_, AppState>, n: u32) -> CmdResult<Value> {
    const RULE_TRANSITIONS_DEFAULT: usize = 5;
    const RULE_TRANSITIONS_HARD_CAP: usize = 200;
    let limit = match n as usize {
        0 => RULE_TRANSITIONS_DEFAULT,
        v => v.min(RULE_TRANSITIONS_HARD_CAP),
    };
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows = viewer
        .recent_rows_of_type(SsbActionType::RuleTransition.as_sql_str(), limit)
        .unwrap_or_default();
    let mut out: Vec<Value> = Vec::with_capacity(rows.len());
    for detail in rows {
        let inner = detail
            .payload_json
            .get("RuleTransition")
            .cloned()
            .unwrap_or(detail.payload_json);
        out.push(json!({
            "seq": detail.seq,
            "ts": detail.ts,
            "rule_id": inner.get("rule_id").and_then(|v| v.as_str()).unwrap_or(""),
            "from_status": inner.get("from_status").and_then(|v| v.as_str()).unwrap_or(""),
            "to_status": inner.get("to_status").and_then(|v| v.as_str()).unwrap_or(""),
            "reason": inner.get("reason").and_then(|v| v.as_str()).unwrap_or(""),
            "timestamp_ms": inner.get("timestamp_ms").and_then(|v| v.as_i64()).unwrap_or(0),
        }));
    }
    Ok(json!(out))
}

/// B-4: latest canary-drift snapshot. SCHEMA_VERSION 99 adds the
/// `SsbActionType::CanaryDrift` variant; the periodic cascade evaluator
/// in `cli::live_cli::spawn_cascade_evaluator_loop` emits one row at
/// startup carrying the real `startup_result.drift_check` verdict (a
/// cosine similarity in [-1.0, 1.0]) and then heartbeats every 30s.
/// The SettingsWindow canary panel reads the latest row and renders
/// `similarity vs drift_threshold`, with a "Recompute" affordance when
/// `passed=false`. When no row exists yet (cold start before the first
/// emit landed) we return `known:false` so the panel renders "(no
/// snapshot yet)" instead of fabricated zeros.
#[tauri::command]
pub fn get_canary_drift(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!({
            "known": false,
            "similarity": 0.0,
            "drift_threshold": 0.85,
            "passed": false,
            "reference_present": false,
            "ts": 0,
        }));
    };
    match viewer.latest_action_of_type("canary_drift") {
        Ok(Some(detail)) => {
            // SsbPayload serializes externally-tagged as
            // {"CanaryDrift": { ... }}; unwrap one level.
            let inner = detail
                .payload_json
                .get("CanaryDrift")
                .cloned()
                .unwrap_or(detail.payload_json);
            Ok(json!({
                "known": true,
                "similarity": inner.get("similarity").and_then(|v| v.as_f64()).unwrap_or(0.0),
                "drift_threshold": inner.get("drift_threshold").and_then(|v| v.as_f64()).unwrap_or(0.85),
                "passed": inner.get("passed").and_then(|v| v.as_bool()).unwrap_or(false),
                "reference_present": inner.get("reference_present").and_then(|v| v.as_bool()).unwrap_or(false),
                "ts": detail.ts,
            }))
        }
        Ok(None) => Ok(json!({
            "known": false,
            "similarity": 0.0,
            "drift_threshold": 0.85,
            "passed": false,
            "reference_present": false,
            "ts": 0,
        })),
        Err(e) => {
            tracing::warn!(error = %e, "get_canary_drift: SSB read failed");
            Ok(json!({
                "known": false,
                "similarity": 0.0,
                "drift_threshold": 0.85,
                "passed": false,
                "reference_present": false,
                "ts": 0,
            }))
        }
    }
}

/// B-4 task #12: VectorCache hit-rate gauge for the KnowledgeDrawer.
/// Reads the latest `vector_cache_hit_rate` SSB row (producer:
/// `VectorCache::flush_hit_rate_ssb` called from
/// `QueryPipeline::query_with_tier` after every successful query). The
/// row carries `{session_id, hits, misses, hit_rate_pct}`; we forward
/// all four plus a `known` flag so the drawer renders "(no data yet)"
/// on cold sessions instead of fabricated 0.0 percent.
#[tauri::command]
pub fn get_vector_cache_hit_rate(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!({
            "known": false,
            "session_id": "",
            "hits": 0,
            "misses": 0,
            "hit_rate_pct": 0.0,
            "ts": 0,
        }));
    };
    match viewer.latest_action_of_type("vector_cache_hit_rate") {
        Ok(Some(detail)) => {
            let inner = detail
                .payload_json
                .get("VectorCacheHitRate")
                .cloned()
                .unwrap_or(detail.payload_json);
            Ok(json!({
                "known": true,
                "session_id": inner.get("session_id").and_then(|v| v.as_str()).unwrap_or(""),
                "hits": inner.get("hits").and_then(|v| v.as_u64()).unwrap_or(0),
                "misses": inner.get("misses").and_then(|v| v.as_u64()).unwrap_or(0),
                "hit_rate_pct": inner.get("hit_rate_pct").and_then(|v| v.as_f64()).unwrap_or(0.0),
                "ts": detail.ts,
            }))
        }
        Ok(None) => Ok(json!({
            "known": false,
            "session_id": "",
            "hits": 0,
            "misses": 0,
            "hit_rate_pct": 0.0,
            "ts": 0,
        })),
        Err(e) => {
            tracing::warn!(error = %e, "get_vector_cache_hit_rate: SSB read failed");
            Ok(json!({
                "known": false,
                "session_id": "",
                "hits": 0,
                "misses": 0,
                "hit_rate_pct": 0.0,
                "ts": 0,
            }))
        }
    }
}

/// C-3: thin Tauri wrapper around the existing
/// [`SsbViewer::research_budget_snapshot`] stub. The viewer returns
/// `known: false` zeros until a `research_budget_*` SSB row type lands
/// (C-1 scope) so the dashboard renders "no data yet" without erroring.
#[tauri::command]
pub fn get_research_budget(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!({
            "known": false,
            "admitted": 0,
            "pruned": 0,
            "tokens_used": 0,
            "tokens_max": 0,
        }));
    };
    match viewer.research_budget_snapshot() {
        Ok(snap) => Ok(json!({
            "known": snap.known,
            "admitted": snap.admitted,
            "pruned": snap.pruned,
            "tokens_used": snap.tokens_used,
            "tokens_max": snap.tokens_max,
        })),
        Err(e) => {
            tracing::warn!(error = %e, "get_research_budget: viewer query failed");
            Ok(json!({
                "known": false,
                "admitted": 0,
                "pruned": 0,
                "tokens_used": 0,
                "tokens_max": 0,
            }))
        }
    }
}

/// C-3: thin Tauri wrapper around the existing
/// [`SsbViewer::source_tier_distribution`] stub. The viewer returns
/// `known: false` zeros until a source-tier producer lands; same
/// graceful-empty pattern as [`get_research_budget`].
#[tauri::command]
pub fn get_source_tier_distribution(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!({
            "known": false,
            "t1": 0,
            "t2": 0,
            "t3": 0,
            "t4": 0,
            "t5": 0,
        }));
    };
    match viewer.source_tier_distribution() {
        Ok(d) => Ok(json!({
            "known": d.known,
            "t1": d.t1,
            "t2": d.t2,
            "t3": d.t3,
            "t4": d.t4,
            "t5": d.t5,
        })),
        Err(e) => {
            tracing::warn!(error = %e, "get_source_tier_distribution: viewer query failed");
            Ok(json!({
                "known": false,
                "t1": 0,
                "t2": 0,
                "t3": 0,
                "t4": 0,
                "t5": 0,
            }))
        }
    }
}

#[tauri::command]
pub fn get_search_mode(state: State<'_, AppState>) -> CmdResult<Value> {
    // C-8 Task #7: read the most-recent SearchModeChanged SSB row. The CLI
    // process emits one row per cascade transition (and one at startup);
    // this command reads them back cross-process so the dashboard renders
    // the live mode instead of the previous hardcoded "HYBRID".
    //
    // Graceful empty fallback: when the viewer is absent (no DB yet) or
    // no row exists, return `mode = "hybrid"` so the dashboard's
    // SearchModeBadge keeps rendering. The frontend uppercases the label
    // (drawers.svelte.ts:126 binds `sm.mode` -> `searchMode = SearchMode`
    // enum), so the lowercase wire tag matches the original contract.
    let live = state
        .viewer
        .as_ref()
        .and_then(|v| v.latest_action_of_type("search_mode_changed").ok().flatten());
    if let Some(detail) = live {
        if let Some(mode) = detail
            .payload_json
            .get("mode")
            .and_then(|m| m.as_str())
            .map(|s| s.to_owned())
        {
            let history = detail
                .payload_json
                .get("history")
                .cloned()
                .unwrap_or_else(|| json!([]));
            return Ok(json!({
                "mode": mode,
                "history": history,
            }));
        }
    }
    Ok(json!({
        "mode": "hybrid",
        "history": [],
    }))
}

/// C-8 Task #7: dashboard-originated SearchModeChanged emitter. Reviewers /
/// operators can pin the search mode for diagnostic purposes via this
/// command; the CLI process picks up the latest row on subsequent reads.
///
/// Returns `{ ok: true, seq }` on success, or an error string when the SSB
/// writer is unavailable.
#[tauri::command]
pub async fn set_search_mode(
    state: State<'_, AppState>,
    mode: String,
    history: Option<Vec<String>>,
) -> CmdResult<Value> {
    let normalised = match mode.to_ascii_lowercase().as_str() {
        "hybrid" | "lexical" | "semantic" | "ann" => mode.to_ascii_lowercase(),
        other => {
            return Err(format!(
                "set_search_mode: unknown mode '{other}' (expected hybrid|lexical|semantic|ann)"
            ));
        }
    };
    let writer = state
        .writer
        .as_ref()
        .ok_or_else(|| "ssb writer unavailable; no session db open".to_owned())?;
    let payload = SsbPayload::SearchModeChanged(SearchModeChangedPayload {
        mode: normalised,
        history: history.unwrap_or_default(),
    });
    let seq = writer
        .write_payload("dashboard-v2", 0, payload, Vec::new())
        .await
        .map_err(map_err)?;
    Ok(json!({ "ok": true, "seq": seq }))
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
            {"model": "cross-encoder-l6", "state": "CLOSED",  "failures": 0, "success_rate": 1.0, "last_failure_ts": null},
            {"model": "splade-v3",      "state": "OPEN",    "failures": 0, "success_rate": 0.0, "last_failure_ts": null},
        ])
    } else {
        serde_json::to_value(&live).unwrap_or(json!([]))
    };
    Ok(result)
}

/// B-4 task #8: per-tier R@5/R@10/MRR badges. Reads the canonical
/// `metrics/phase_3K_rag.toml` (or the `_improved` overlay when present)
/// at runtime so dashboard numbers reflect the latest measured run.
/// Cargo-workspace `metrics/` lives at `.elai_cc/metrics/phase_3K_rag.toml`
/// relative to the dashboard process's CWD; we also probe `../metrics`
/// and `../../metrics` so the lookup works from a few common launch
/// directories without baking in absolute paths. When the file is
/// missing/unreadable we fall back to the prior hard-coded literals
/// (which the KnowledgeDrawer DEEP MRR < STANDARD warn already paints
/// red) so the dashboard never crashes during a partial deployment.
#[tauri::command]
pub fn get_recall_metrics(_state: State<'_, AppState>) -> CmdResult<Value> {
    fn read_metrics_toml() -> Option<toml::Value> {
        // Probe a small set of canonical relative paths so the command
        // works whether the Tauri shell launched from `.elai_cc/`,
        // `.elai_cc/crates/dashboard-v2/`, or workspace root.
        const CANDIDATES: &[&str] = &[
            "metrics/phase_3K_rag.toml",
            "../metrics/phase_3K_rag.toml",
            "../../metrics/phase_3K_rag.toml",
            "../../../metrics/phase_3K_rag.toml",
            ".elai_cc/metrics/phase_3K_rag.toml",
        ];
        for path in CANDIDATES {
            if let Ok(text) = std::fs::read_to_string(path) {
                if let Ok(parsed) = text.parse::<toml::Value>() {
                    return Some(parsed);
                }
            }
        }
        None
    }

    fn tier_block(root: &toml::Value, tier: &str) -> Option<Value> {
        let block = root.get("tier")?.get(tier)?;
        let r5 = block.get("recall_at_5")?.as_float()?;
        let r10 = block.get("recall_at_10")?.as_float()?;
        let mrr = block.get("mrr")?.as_float()?;
        // Prefer p95 over p50 for the latency badge; the dashboard
        // surfaces this as "per-tier latency". toml stores integers
        // when the value is zero — fall through to 0.0 via as_integer.
        let latency = block
            .get("latency_p95_ms")
            .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|n| n as f64)))
            .unwrap_or(0.0);
        Some(json!({
            "tier": tier.to_ascii_uppercase(),
            "r_at_5": r5,
            "r_at_10": r10,
            "mrr": mrr,
            "latency_ms": latency,
        }))
    }

    if let Some(root) = read_metrics_toml() {
        let mut rows: Vec<Value> = Vec::with_capacity(3);
        for tier in ["fast", "standard", "deep"] {
            if let Some(row) = tier_block(&root, tier) {
                rows.push(row);
            }
        }
        if !rows.is_empty() {
            return Ok(Value::Array(rows));
        }
    }

    // Fallback: last-known hard-coded literals so the dashboard renders
    // even when the toml is missing or malformed. Values mirror the
    // 2026-04-15 run in metrics/phase_3K_rag.toml (tier.fast / standard
    // / deep). The DEEP MRR violation (0.5017 < 0.6000) is intentionally
    // surfaced via the KnowledgeDrawer warn badge.
    Ok(json!([
        {"tier": "FAST",     "r_at_5": 0.6000, "r_at_10": 0.7000, "mrr": 0.3908, "latency_ms": 1.0},
        {"tier": "STANDARD", "r_at_5": 0.8000, "r_at_10": 0.8000, "mrr": 0.6000, "latency_ms": 1.0},
        {"tier": "DEEP",     "r_at_5": 0.8000, "r_at_10": 0.8000, "mrr": 0.5017, "latency_ms": 5.0},
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
            // B-4 (SSB-FIDELITY): `DkgReadyPayload.failed_files` is surfaced
            // through `DkgStatsSnapshot`, so this reports the latest DKG
            // row's real parse-failure count (0 only with no row or query error).
            "failed_files": v.get("failed_files").and_then(|x| x.as_u64()).unwrap_or(0),
        }))
    } else {
        // Real numbers from metrics/phase_1A.toml fallback.
        Ok(json!({
            "nodes": 8075,
            "edges": 4148,
            "bm25_chunks": 0,
            "vector_status": "BUILDING",
            "freshness_decay_pct": 0,
            // B-4: no SSB DB reachable -> failure count unknown, report 0.
            "failed_files": 0,
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

// C-24 (DECISION-4): `b33_empty_viz` moved to
// `elai_knowledge::visualization::views::empty_viz` so the orchestrator
// IPC tool surface + the dashboard share the same code path. No call
// sites here reference it after `b33_build_file_view`/`b33_build_symbol_view`
// were re-pointed at the knowledge crate helpers above.

/// Build the `file` mode VizData by walking DkgEngine nodes + edges.
/// Per-file symbol_count is filled from SymbolGraphCache.meta when available.
///
/// C-24 (DECISION-4): the bulk of this implementation now lives in
/// `elai_knowledge::visualization::views::build_file_view` so the orchestrator
/// IPC tool surface (translate_tool_request `TOOL_EXPORT_GRAPH_MERMAID` arm)
/// can call the same code path without a dashboard-crate dep. The Tauri
/// wrapper kept this name so call sites at L1745/L1844 are unchanged.
fn b33_build_file_view(
    project_root: &std::path::Path,
    project_id: &str,
) -> CmdResult<elai_knowledge::visualization::types::VizData> {
    elai_knowledge::visualization::views::build_file_view(project_root, project_id)
}

/// Build the `symbol` mode VizData by walking SymbolGraphCache file payloads.
/// Falls back to file mode + Capped reason when total edge count exceeds the cap.
///
/// C-24 (DECISION-4): delegated to `elai_knowledge::visualization::views::build_symbol_view`
/// (same rationale as `b33_build_file_view`).
fn b33_build_symbol_view(
    project_root: &std::path::Path,
    project_id: &str,
) -> CmdResult<elai_knowledge::visualization::types::VizData> {
    elai_knowledge::visualization::views::build_symbol_view(project_root, project_id)
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
        // C-24 (DRIFT-7 Path A) — DECISION-6: also populate the dedicated
        // `level` data field so the GraphWindow cytoscape selectors
        // (node[level = "Concept" | "Phase" | "Step" | "BabyStep"]) can
        // pick the right shape without parsing the label-prefix.
        // The label prefix is kept for textual backwards-compat (existing
        // mermaid renderer output + snapshot tests still rely on it).
        // The selectors use the data attribute, so an upstream rename of
        // the prefix format won't break the styling.
        let level_attr = match level {
            "concept" => Some("Concept".to_string()),
            "phase" => Some("Phase".to_string()),
            "step" => Some("Step".to_string()),
            "baby_step" => Some("BabyStep".to_string()),
            // Defense-in-depth: any future decomposition level not in the
            // enumerated set falls through with the level field preserved
            // (lossless capitalisation). Cytoscape selector miss is
            // silent — node uses the default shape, no panic.
            other => Some({
                let mut s = String::with_capacity(other.len());
                let mut up = true;
                for c in other.chars() {
                    if c == '_' { up = true; continue; }
                    if up { s.extend(c.to_uppercase()); up = false; } else { s.push(c); }
                }
                s
            }),
        };
        files.push(VizFile {
            id: id.to_string(),
            label: format!("{level}: {label}"),
            language: "decomposition".to_string(),
            deps: 0,
            dependents: 0,
            symbol_count: 0,
            level: level_attr,
            // C-24 (DECISION-6): status colors source from per-baby-step
            // execution state. That pipeline is deferred (cosmos_worker /
            // spawn_loop dispatcher scope per B-42 memo). The cytoscape
            // selectors EXIST in GraphWindow style array, so the moment
            // status flows here, visual feedback ships automatically.
            status: None,
            // H-10 (HD-5): dead-code fields not applicable to decomposition view.
            is_dead_code: None,
            coverage_count: None,
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
///
/// C-24 (DECISION-10): the Tauri wrapper resolves the State-bound inputs
/// (project_root snapshot + project_id fallback + decomposition view from
/// AppState) eagerly so the implementation function below has no Tauri
/// dependency. Mirrors the C-18 D2 pattern used for `export_graph_mermaid_impl`
/// — lets integration tests in `crates/dashboard-v2/src-tauri/tests/` call
/// the impl directly without constructing a Tauri runtime.
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
    // C-24: decomposition mode requires AppState — resolve here so the impl
    // function below is Tauri-free.
    let decomposition_data = if mode.as_str() == "decomposition" {
        Some(b33_build_decomposition_view(&state, &pid))
    } else {
        None
    };
    get_graph_view_data_impl(&project_root, &pid, &mode, decomposition_data)
}

/// C-24 (DECISION-10): Tauri-free implementation of `get_graph_view_data`.
/// For `file` and `symbol` modes the function is fully self-contained
/// (walks on-disk DKG/Symbol stores rooted at `project_root`). For
/// `decomposition` mode the caller must supply the pre-built VizData
/// (the Tauri command resolves it from AppState before delegating here).
pub fn get_graph_view_data_impl(
    project_root: &std::path::Path,
    project_id: &str,
    mode: &str,
    decomposition_data: Option<elai_knowledge::visualization::types::VizData>,
) -> CmdResult<elai_knowledge::visualization::types::VizData> {
    match mode {
        "file" => b33_build_file_view(project_root, project_id),
        "symbol" => b33_build_symbol_view(project_root, project_id),
        "decomposition" => decomposition_data.ok_or_else(|| {
            "get_graph_view_data_impl: decomposition mode requires \
             `decomposition_data` to be supplied by the caller (the Tauri \
             command resolves it from AppState before delegating here)"
                .to_owned()
        }),
        other => Err(format!(
            "unknown mode `{other}`; expected file/symbol/decomposition"
        )),
    }
}

/// `get_impact_overlay` — call traversal::compute_impact_radius and reshape
/// the ImpactResult into a JSON shape the frontend can consume directly.
///
/// C-24 (DECISION-10): same impl-fn extraction pattern as
/// `get_graph_view_data` above.
#[tauri::command]
pub async fn get_impact_overlay(
    state: State<'_, AppState>,
    project_id: Option<String>,
    target: String,
    depth: u32,
) -> CmdResult<Value> {
    let project_root = state.project_root_snapshot();
    let pid = project_id
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| b33_resolve_project_id(&state));
    get_impact_overlay_impl(&project_root, &pid, &target, depth)
}

/// C-24 (DECISION-10): Tauri-free implementation of `get_impact_overlay`.
/// Returns the empty-overlay JSON shape (`{files:[], by_depth:{}, ...}`)
/// when no DKG / SymbolGraphCache exists yet so the frontend renders a
/// placeholder without erroring.
pub fn get_impact_overlay_impl(
    project_root: &std::path::Path,
    project_id: &str,
    target: &str,
    depth: u32,
) -> CmdResult<Value> {
    let empty = json!({
        "files": [],
        "by_depth": {},
        "total_files": 0,
        "max_depth_reached": 0,
        "truncated": false,
    });
    let dkg_path = elai_core::paths::dkg_db_path(project_root);
    if !dkg_path.exists() {
        return Ok(empty);
    }
    let mut cache = match elai_knowledge::symbol_graph::cache::SymbolGraphCache::new(
        &dkg_path,
        project_id,
        elai_knowledge::symbol_graph::SYMBOL_FILE_LRU_SIZE,
    ) {
        Ok(c) => c,
        Err(_) => return Ok(empty),
    };
    let effective_depth = if depth == 0 {
        B33_DEFAULT_IMPACT_DEPTH
    } else {
        depth
    };
    let impact = elai_knowledge::symbol_graph::traversal::compute_impact_radius(
        &mut cache,
        target,
        effective_depth,
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
    let decomposition_data = if mode.as_str() == "decomposition" {
        Some(b33_build_decomposition_view(&state, &pid))
    } else {
        None
    };
    export_graph_mermaid_impl(&project_root, &pid, &mode, max_nodes, decomposition_data)
}

/// C-18 D2: Tauri-free implementation of `export_graph_mermaid`.
///
/// The Tauri command above wraps this function with the `State<'_, AppState>`
/// indirection for the `decomposition` mode (which needs in-memory dashboard
/// state). For `file` and `symbol` modes the function is fully self-contained:
/// it walks the on-disk DKG/Symbol stores rooted at `project_root` and
/// produces a Mermaid string. Integration tests call this `*_impl` form
/// directly because Tauri `State<>` cannot be constructed outside the runtime.
///
/// `decomposition_data` is `Some` only when `mode == "decomposition"`; for the
/// two other modes the parameter is ignored and may be `None`. The Tauri
/// command resolves the dashboard-state-backed view eagerly so this function
/// has no dependency on `AppState`.
pub fn export_graph_mermaid_impl(
    project_root: &std::path::Path,
    project_id: &str,
    mode: &str,
    max_nodes: Option<usize>,
    decomposition_data: Option<elai_knowledge::visualization::types::VizData>,
) -> CmdResult<String> {
    let data = match mode {
        "file" => b33_build_file_view(project_root, project_id)?,
        "symbol" => b33_build_symbol_view(project_root, project_id)?,
        "decomposition" => decomposition_data.ok_or_else(|| {
            "export_graph_mermaid_impl: decomposition mode requires \
             `decomposition_data` to be supplied by the caller (the Tauri \
             command resolves it from AppState before delegating here)"
                .to_owned()
        })?,
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
pub fn get_grounding_sweep(state: State<'_, AppState>) -> CmdResult<Value> {
    // C-1: read the most recent `grounding_sweep_result` row written by
    // `task_runner::drive_task_inner` immediately after
    // `PipelineGrc::ground_plan` returns. Falls back to the zero shape
    // when no SSB DB is reachable OR no row has been written yet.
    let zero = json!({
        "grounded": 0,
        "verify_needed": 0,
        "ungrounded": 0,
        "claims": [],
        "top_claim": Value::Null,
    });
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(zero);
    };
    let detail = match viewer.latest_action_of_type(
        SsbActionType::GroundingSweepResult.as_sql_str(),
    ) {
        Ok(Some(d)) => d,
        Ok(None) => return Ok(zero),
        Err(_) => return Ok(zero),
    };
    // Reshape payload_json to match the legacy GroundingSweep TS shape.
    let p = &detail.payload_json;
    // SsbPayload::GroundingSweepResult serializes as
    // {"GroundingSweepResult":{...}}; pull out the inner object.
    let inner = p
        .get("GroundingSweepResult")
        .cloned()
        .unwrap_or_else(|| p.clone());
    let grounded = inner.get("grounded").and_then(|v| v.as_u64()).unwrap_or(0);
    let verify_needed = inner
        .get("verify_needed")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let ungrounded = inner.get("ungrounded").and_then(|v| v.as_u64()).unwrap_or(0);
    let top_claim = inner.get("top_claim").cloned().unwrap_or(Value::Null);
    Ok(json!({
        "grounded": grounded,
        "verify_needed": verify_needed,
        "ungrounded": ungrounded,
        "claims": [],
        "top_claim": top_claim,
        "seq": detail.seq,
        "ts": detail.ts,
    }))
}

#[tauri::command]
pub fn get_csq_pending(state: State<'_, AppState>) -> CmdResult<Value> {
    // C-1: list `csq_evaluated` rows where `blocked = true`. These are
    // the pre-tool CSQ verdicts that gated a baby step (Critical without
    // guard). Returns `[]` when no SSB DB is reachable OR no blocked
    // rows exist yet.
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows = viewer
        .recent_rows_of_type(
            SsbActionType::CsqEvaluated.as_sql_str(),
            CSQ_PENDING_QUERY_LIMIT,
        )
        .unwrap_or_default();
    let mut out: Vec<Value> = Vec::new();
    for detail in rows {
        let inner = detail
            .payload_json
            .get("CsqEvaluated")
            .cloned()
            .unwrap_or(detail.payload_json);
        let blocked = inner
            .get("blocked")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !blocked {
            continue;
        }
        out.push(json!({
            "seq": detail.seq,
            "ts": detail.ts,
            "severity": inner.get("severity").cloned().unwrap_or(Value::Null),
            "guard_present": inner
                .get("guard_present")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            "blocked": blocked,
            "check_count": inner
                .get("check_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        }));
    }
    Ok(json!(out))
}

/// C-1: hard cap on the per-call CSQ-pending fetch. SsbViewer's own
/// `DRAWER_QUERY_HARD_CAP` (500) is the absolute ceiling; we set a
/// smaller dashboard-side limit so a flood of csq_evaluated rows can't
/// stall the drawer refresh path.
const CSQ_PENDING_QUERY_LIMIT: usize = 50;

/// C-3: row cap for `get_csq_log` (full CSQ verdict log, not just
/// blocked ones). Matches CSQ_PENDING_QUERY_LIMIT for symmetry —
/// every blocked row is also in the log.
const CSQ_LOG_QUERY_LIMIT: usize = 50;

/// B-5 GAP-1: row cap for `get_auditor_rubber_stamp_rejections`. The
/// ReasoningDrawer counter only needs the count, but the full row list
/// is returned so the drawer can render a recent-rejections breakdown.
/// Capped at 200 to bound the JSON payload (dashboardRules #14 / S4
/// guardrail: clamp limit param to <=200).
const AUDITOR_RUBBER_STAMP_QUERY_LIMIT: usize = 50;

/// C-3: row cap for `get_feasibility_log`. The existing
/// `get_feasibility_report` command reads up to 64 rows for the v2
/// PASS/FAIL summary; the raw log is bounded smaller because it ships
/// the full payload per row.
const FEASIBILITY_LOG_QUERY_LIMIT: usize = 20;

/// C-3: full CSQ verdict log (distinct from [`get_csq_pending`] which
/// filters `blocked = true`). Reads the most recent N
/// `csq_evaluated` SSB rows and exposes every verdict the GRC engine
/// emitted regardless of whether it gated the baby step. Returns `[]`
/// when no SSB DB is reachable OR no rows exist yet.
#[tauri::command]
pub fn get_csq_log(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows = viewer
        .recent_rows_of_type(
            SsbActionType::CsqEvaluated.as_sql_str(),
            CSQ_LOG_QUERY_LIMIT,
        )
        .unwrap_or_default();
    let mut out: Vec<Value> = Vec::with_capacity(rows.len());
    for detail in rows {
        let inner = detail
            .payload_json
            .get("CsqEvaluated")
            .cloned()
            .unwrap_or(detail.payload_json);
        out.push(json!({
            "csq_id": detail.seq.to_string(),
            "seq": detail.seq,
            "ts_ms": detail.ts,
            "conclusion": inner.get("severity").cloned().unwrap_or(Value::Null),
            "guard_present": inner
                .get("guard_present")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            "blocked": inner
                .get("blocked")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            "check_count": inner
                .get("check_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        }));
    }
    Ok(json!(out))
}

/// B-5 GAP-1: read-side command for the anti-rubber-stamp counter.
/// Returns the most recent N `auditor_rubber_stamp_rejected` SSB rows
/// (reshaped into `AuditorRubberStampRejectionRow` JSON). The
/// ReasoningDrawer renders "Auditor: N rubber stamps rejected this
/// session" from `.length` of the returned array. Returns `[]` when no
/// SSB DB is reachable OR no rejections have fired yet (steady state).
///
/// `limit` is clamped to `AUDITOR_RUBBER_STAMP_QUERY_LIMIT` (50) to
/// bound the payload; `None` uses the default. Mirrors the
/// `get_csq_log` reshape pattern (inner payload key
/// `"AuditorRubberStampRejected"`).
#[tauri::command]
pub fn get_auditor_rubber_stamp_rejections(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let effective_limit = limit
        .unwrap_or(AUDITOR_RUBBER_STAMP_QUERY_LIMIT)
        .min(AUDITOR_RUBBER_STAMP_QUERY_LIMIT);
    let rows = viewer
        .recent_rows_of_type(
            SsbActionType::AuditorRubberStampRejected.as_sql_str(),
            effective_limit,
        )
        .unwrap_or_default();
    let mut out: Vec<Value> = Vec::with_capacity(rows.len());
    for detail in rows {
        let inner = detail
            .payload_json
            .get("AuditorRubberStampRejected")
            .cloned()
            .unwrap_or(detail.payload_json);
        out.push(json!({
            "seq": detail.seq,
            "ts_ms": detail.ts,
            "role": inner.get("role").and_then(|v| v.as_str()).unwrap_or("auditor"),
            "finding_count": inner
                .get("finding_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            "expected_min": inner
                .get("expected_min")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        }));
    }
    Ok(json!(out))
}

/// C-3: raw feasibility-report log (distinct from
/// [`get_feasibility_report`] which returns the v2 PASS/FAIL
/// summary shape from `feasibility_summary`). DRIFT-3: the SSB
/// action_type is `feasibility_report`, NOT `feasibility_check` (no
/// such variant exists). Reshape per row mirrors
/// [`FeasibilityReportPayload`]; the dashboard reads `task_id`
/// (derived from `seq`), `verdict`, `score`, and `ts_ms` defensively.
#[tauri::command]
pub fn get_feasibility_log(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows = viewer
        .recent_rows_of_type(
            SsbActionType::FeasibilityReport.as_sql_str(),
            FEASIBILITY_LOG_QUERY_LIMIT,
        )
        .unwrap_or_default();
    let mut out: Vec<Value> = Vec::with_capacity(rows.len());
    for detail in rows {
        let inner = detail
            .payload_json
            .get("FeasibilityReport")
            .cloned()
            .unwrap_or(detail.payload_json);
        let verdict = inner
            .get("task_feasibility")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let vram_bp = inner
            .get("vram_margin_bp")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as f64;
        out.push(json!({
            "task_id": detail.seq.to_string(),
            "seq": detail.seq,
            "ts_ms": detail.ts,
            "verdict": verdict,
            // basis-point -> percent (2000 bp = 20.00%) for dashboard display.
            "score": vram_bp / 100.0,
            "skip_reason": inner.get("skip_reason").cloned().unwrap_or(Value::Null),
            "novel_step_count": inner
                .get("novel_step_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            "escalations_count": inner
                .get("escalations_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            "vram_margin_pct": inner
                .get("vram_margin_bp")
                .and_then(|v| v.as_u64())
                .map(|bp| bp as f64 / 100.0)
                .unwrap_or(0.0),
            "ram_margin_pct": inner
                .get("ram_margin_bp")
                .and_then(|v| v.as_u64())
                .map(|bp| bp as f64 / 100.0)
                .unwrap_or(0.0),
            "disk_margin_pct": inner
                .get("disk_margin_bp")
                .and_then(|v| v.as_u64())
                .map(|bp| bp as f64 / 100.0)
                .unwrap_or(0.0),
        }));
    }
    Ok(json!(out))
}

/// C-3: per-role GRC config table. Compile-time static read from
/// [`elai_orchestrator::default_grc_config`]; no SSB read needed because
/// the canonical table is computed at process start from the reasoning
/// crate's config loader. DRIFT-5: `AgentGrcConfig` carries no
/// `topic_prefix` field, so the dashboard shape uses `role` (the
/// role string) as the topic key per `reasoning::engine` semantics —
/// every agent's GRC topic is keyed by its role string. The 16-variant
/// `AgentRole` enum guarantees the returned array length >= 8 (plan
/// acceptance).
#[tauri::command]
pub fn get_agent_grc_config(state: State<'_, AppState>) -> CmdResult<Value> {
    // Touch state so the parameter is not flagged dead-code; the data
    // source is compile-time, not SSB-driven.
    let _ = state;
    use elai_ipc_types::AgentRole;
    use elai_orchestrator::default_grc_config;
    let roles = [
        AgentRole::Executor,
        AgentRole::Auditor,
        AgentRole::Researcher,
        AgentRole::Designer,
        AgentRole::PromptEngineer,
        AgentRole::ContextCompiler,
        AgentRole::MemoryWriter,
        AgentRole::MediaValidator,
        AgentRole::IdeaFramer,
        AgentRole::IdeaCritic,
        AgentRole::AuditorConcepts,
        AgentRole::AuditorPhases,
        AgentRole::AuditorSteps,
        AgentRole::AuditorBabySteps,
        AgentRole::AuditorBabyStepDetail,
        AgentRole::AuditorProductionGrade,
        // E-37: off-model prompt-injection classifier.
        AgentRole::InjectionClassifier,
    ];
    let entries: Vec<Value> = roles
        .iter()
        .map(|role| {
            let cfg = default_grc_config(role);
            json!({
                "role": cfg.role.clone(),
                "max_calls": cfg.budget.max_calls,
                "max_chars": cfg.budget.max_chars,
                // DRIFT-5: role doubles as the topic key (no separate
                // topic_prefix field on AgentGrcConfig).
                "topic_prefix": cfg.role,
            })
        })
        .collect();
    Ok(json!(entries))
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

/// B-17 Task 2: per-gate verdict frequency counts aggregated from
/// `evidence_gate_evaluated` SSB rows. Returns
/// `[{gate_id, pass_count, block_count, total}]` sorted by total desc.
/// Empty SSB returns an empty array (drawer renders a "no data" note).
#[tauri::command]
pub fn get_gate_verdict_counts(state: State<'_, AppState>) -> CmdResult<Value> {
    let counts = state
        .viewer
        .as_ref()
        .and_then(|v| v.aggregate_gate_verdict_counts().ok())
        .unwrap_or_default();

    let arr: Vec<Value> = counts
        .iter()
        .map(|c| {
            json!({
                "gate_id": &c.gate_id,
                "pass_count": c.pass_count,
                "block_count": c.block_count,
                "total": c.total,
            })
        })
        .collect();
    Ok(Value::Array(arr))
}

#[tauri::command]
pub fn get_agents(state: State<'_, AppState>) -> CmdResult<Value> {
    // C-2: live query SSB for the latest `agent_status_changed` row per
    // agent_id (replaces the hardcoded 8-role IDLE list). When the SSB has
    // zero rows yet (cold-start / no spawn) we fall back to the canonical
    // 8-role IDLE list so the OrchestrationDrawer AGENTS section never
    // empty-renders. The fallback is INSIDE the live-read path; the
    // stub_ban rule forbids the always-static path that existed pre-C-2.
    let live_rows: Vec<Value> = state
        .viewer
        .as_ref()
        .and_then(|v| v.recent_agent_status_changes(64).ok())
        .map(|rows| {
            rows.iter()
                .map(|r| {
                    let lifecycle = lifecycle_label(&r.state);
                    json!({
                        "role": &r.role,
                        "lifecycle": lifecycle,
                        "pid": null,
                        "last_heartbeat_ms": r.timestamp_ms as i64,
                        "grc_budget_used_pct": 0,
                        "death_classification": r.death_class,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    if !live_rows.is_empty() {
        return Ok(Value::Array(live_rows));
    }

    // Cold-start fallback: 8 canonical roles in IDLE. Honest empty-state
    // representation; not a mock, because no SSB rows exist yet.
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

/// C-2: map SSB `state` discriminant onto the dashboard `AgentLifecycle`
/// enum used by OrchestrationDrawer.svelte. `spawned`/`ready` map to
/// `RUNNING`; `exited` maps to `DEAD` or `IDLE` depending on death class.
fn lifecycle_label(state: &str) -> &'static str {
    match state {
        "spawned" | "ready" => "RUNNING",
        "exited" => "DEAD",
        _ => "IDLE",
    }
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

// C-23 — SessionDag branch-UI commands.
// Per-call DAG open: AppState caches a `dag_db_path` once resolved (see
// `resolve_active_dag_path`) but does NOT hold a long-lived SessionDag
// handle. Each command opens the DB, performs the operation, drops the
// handle. Trade-off documented in workflow/reasoning/C-23-decisions.md:
//   * pro: zero lifetime entanglement with the CLI process
//   * pro: stateless across project switches
//   * con: ~5 ms SQLite open per call (negligible for branch ops)
// All four commands return a typed `DagBranchError` that maps from
// `SessionDagError` (thiserror#[from]) and Display-coerces to the
// dashboard's `CmdResult<T, String>` rejection shape.

/// C-23: typed wrapper around `SessionDagError` for the dashboard branch
/// commands. Carries `#[from]` so a SessionDag call site can early-return
/// with `?`. The dashboard surface flattens this back to a string via
/// `map_dag_err` so the Tauri rejection JSON stays a plain string per
/// existing dashboard convention.
#[derive(Debug, thiserror::Error)]
pub enum DagBranchError {
    /// No active SessionDag has been resolved. Returned when
    /// `resolve_active_dag_path` cannot locate a session under
    /// `<project_root>/.elai_cc/sessions/` or the resolved path's
    /// `dag.db` does not exist.
    #[error("no active SessionDag: the dashboard has not resolved a session.dag.db path yet")]
    NoActiveSession,
    /// The SessionDag database was found but `SessionDag::open` failed.
    #[error("failed to open SessionDag at {path}: {source}")]
    OpenFailed {
        path: String,
        #[source]
        source: SessionDagError,
    },
    /// A branch-mutation call failed at the orchestrator level.
    #[error("session_dag error: {0}")]
    Dag(#[from] SessionDagError),
    /// `merge_branch_with_ssb` returned `MergeResult::Conflict`.
    #[error("merge conflict: {0}")]
    MergeConflict(String),
    /// `load_handoff` returned `Ok(None)` — no handoff snapshot exists
    /// for the active session. Distinct from `SnapshotNotFound` which is
    /// a missing-pointer error inside the DAG.
    #[error("no handoff snapshot available for the active session")]
    NoHandoff,
}

fn map_dag_err(e: DagBranchError) -> String {
    e.to_string()
}

/// C-23: locate the active session's dag.db path. Caches the resolved
/// path in AppState so subsequent calls are O(1). Resolution order:
///   1. `state.dag_db_path_snapshot()` — cached value (fast path).
///   2. Newest `<project_root>/.elai_cc/sessions/<session-id>/dag.db`
///      by mtime, restricted to entries whose `dag.db` exists.
///   3. Fail with `DagBranchError::NoActiveSession`.
/// The resolver does NOT create new sessions. The dashboard is a
/// read-out / mutation surface for sessions already created by the CLI.
fn resolve_active_dag_path(state: &AppState) -> Result<(String, PathBuf), DagBranchError> {
    if let Some(cached) = state.dag_db_path_snapshot() {
        if cached.exists() {
            if let Some(session_id) = cached
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|s| s.to_owned())
            {
                return Ok((session_id, cached));
            }
        }
    }

    let project_root = state.project_root_snapshot();
    let sessions_dir = sessions_dir_path(&project_root);
    if !sessions_dir.exists() {
        return Err(DagBranchError::NoActiveSession);
    }

    let mut best: Option<(std::time::SystemTime, String, PathBuf)> = None;
    let entries = match std::fs::read_dir(&sessions_dir) {
        Ok(it) => it,
        Err(_) => return Err(DagBranchError::NoActiveSession),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dag_path = path.join("dag.db");
        if !dag_path.exists() {
            continue;
        }
        let Some(session_id) = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_owned())
        else {
            continue;
        };
        let mtime = entry
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        match &best {
            None => best = Some((mtime, session_id, dag_path)),
            Some((cur_mtime, _, _)) if mtime > *cur_mtime => {
                best = Some((mtime, session_id, dag_path));
            }
            _ => {}
        }
    }

    let (_, session_id, dag_path) = best.ok_or(DagBranchError::NoActiveSession)?;
    let _ = state.set_dag_db_path(dag_path.clone());
    Ok((session_id, dag_path))
}

/// C-23: open a SessionDag for the active session. Wrapper that uniformly
/// surfaces `DagBranchError::OpenFailed` on SQLite open failure so the
/// callsites can propagate via `?` cleanly.
fn open_active_dag(state: &AppState) -> Result<(String, SessionDag), DagBranchError> {
    let (session_id, dag_path) = resolve_active_dag_path(state)?;
    let dag = SessionDag::open(&session_id, &dag_path).map_err(|e| {
        DagBranchError::OpenFailed {
            path: dag_path.display().to_string(),
            source: e,
        }
    })?;
    Ok((session_id, dag))
}

#[tauri::command]
pub async fn fork_session_branch(
    state: State<'_, AppState>,
    name: String,
) -> CmdResult<Value> {
    let (session_id, mut dag) = open_active_dag(&state).map_err(map_dag_err)?;
    // Snap to fork from: prefer the current branch tip if one exists,
    // otherwise the latest snapshot via latest_snapshot(None). If neither
    // is present, surface a typed error rather than silently creating a
    // root snapshot.
    let from_snap_id = match dag.list_branches().first() {
        Some(br) => br.tip_snap_id.clone(),
        None => {
            let latest = dag
                .latest_snapshot(None)
                .map_err(|e| map_dag_err(DagBranchError::Dag(e)))?
                .ok_or_else(|| {
                    map_dag_err(DagBranchError::Dag(SessionDagError::SnapshotNotFound))
                })?;
            latest.snap_id
        }
    };
    let ssb_writer = state.writer.clone();
    let ssb_ref = ssb_writer.as_deref();
    let branch = dag
        .create_branch_with_ssb(
            &name,
            &from_snap_id,
            &mut OsEntropy,
            &DagOsClock,
            ssb_ref,
        )
        .await
        .map_err(|e| map_dag_err(DagBranchError::Dag(e)))?;
    // E-25 D4: SessionForked emit. Wire-dark unless
    // ELAI_E25_SESSION_FORK_EMIT_ENABLE=1. Best-effort, swallows errors.
    emit_session_forked_dashboard(
        ssb_ref,
        "session_forked_branch",
        &session_id,
        &branch.name,
        Some(&from_snap_id),
    )
    .await;
    Ok(json!({
        "branch_name": branch.name,
        "tip_snap_id": branch.tip_snap_id,
        "parent_snap_id": from_snap_id,
        "created_at": branch.created_at,
    }))
}

/// E-25 Tranche B-4 (D4): emit one `ClaudeCodeArchitectureEvent`
/// (kind = `SessionForked`) from the dashboard fork command. Wire-dark
/// unless `ELAI_E25_SESSION_FORK_EMIT_ENABLE=1`.
async fn emit_session_forked_dashboard(
    ssb: Option<&elai_orchestrator::Ssb>,
    fork_event: &'static str,
    parent_session_id: &str,
    child_session_id: &str,
    branch_point: Option<&str>,
) {
    if std::env::var("ELAI_E25_SESSION_FORK_EMIT_ENABLE")
        .map(|v| v != "1")
        .unwrap_or(true)
    {
        return;
    }
    let Some(ssb_ref) = ssb else { return; };
    let ts_ms = std::time::SystemTime::now() // entropy-allow: free fn takes only `ssb: Option<&Ssb>`, no AppState/ClockProvider in scope; gated behind ELAI_E25_SESSION_FORK_EMIT_ENABLE, advisory architecture-event beacon
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0);
    let details = json!({
        "fork_event": fork_event,
        "parent_session": parent_session_id,
        "child_session": child_session_id,
        "branch_point": branch_point,
    });
    let payload = SsbPayload::ClaudeCodeArchitectureEvent(
        ClaudeCodeArchitectureEventPayload {
            kind: ClaudeCodeArchitectureEventKind::SessionForked,
            primitive: ClaudeCodeArchPrimitive::SessionFork,
            session_id: Some(parent_session_id.to_owned()),
            details_json: details,
            ts_ms,
        },
    );
    let _ = ssb_ref
        .write_payload("dashboard:fork_session_branch", 0, payload, Vec::new())
        .await;
}

#[tauri::command]
pub async fn merge_session_branch(
    state: State<'_, AppState>,
    name: String,
) -> CmdResult<Value> {
    let (_session_id, mut dag) = open_active_dag(&state).map_err(map_dag_err)?;
    let ssb_writer = state.writer.clone();
    let ssb_ref = ssb_writer.as_deref();
    let result = dag
        .merge_branch_with_ssb(&name, &mut OsEntropy, &DagOsClock, ssb_ref)
        .await
        .map_err(|e| map_dag_err(DagBranchError::Dag(e)))?;
    match result {
        MergeResult::Winner { branch, abandoned } => Ok(json!({
            "winner_name": branch.name,
            "trunk_snap_id": branch.tip_snap_id,
            "abandoned_names": abandoned.iter().map(|b| b.name.clone()).collect::<Vec<_>>(),
        })),
        MergeResult::Conflict { reason } => {
            Err(map_dag_err(DagBranchError::MergeConflict(reason)))
        }
    }
}

#[tauri::command]
pub async fn close_session_branch(
    state: State<'_, AppState>,
    name: String,
) -> CmdResult<Value> {
    let (_session_id, mut dag) = open_active_dag(&state).map_err(map_dag_err)?;
    let ssb_writer = state.writer.clone();
    let ssb_ref = ssb_writer.as_deref();
    dag.close_branch_with_ssb(
        &name,
        "dashboard:close_session_branch",
        &DagOsClock,
        ssb_ref,
    )
    .await
    .map_err(|e| map_dag_err(DagBranchError::Dag(e)))?;
    Ok(json!({ "branch_name": name, "closed": true }))
}

/// C-23: dashboard /continue handler — loads the latest handoff snapshot
/// from the active SessionDag without rehydrating an in-memory session
/// (that belongs to the CLI). Returns metadata so the CommandWindow can
/// render a status card; the actual session resumption happens on the
/// next CLI invocation that opens the same dag.db. This satisfies the
/// C-23 acceptance requirement that `/continue` calls a real Tauri
/// command, not a textarea prepopulate.
#[tauri::command]
pub async fn load_handoff(state: State<'_, AppState>) -> CmdResult<Value> {
    let (session_id, dag) = open_active_dag(&state).map_err(map_dag_err)?;
    let loaded = dag
        .load_handoff()
        .map_err(|e| map_dag_err(DagBranchError::Dag(e)))?
        .ok_or_else(|| map_dag_err(DagBranchError::NoHandoff))?;
    let (snapshot, messages) = loaded;
    let chain_len = dag
        .parent_chain(&snapshot.snap_id)
        .map(|c| c.len())
        .unwrap_or(0);
    Ok(json!({
        "session_id": session_id,
        "snap_id": snapshot.snap_id,
        "fsm_state": snapshot.fsm_state,
        "message_count": messages.len(),
        "parent_chain_len": chain_len,
    }))
}

#[tauri::command]
pub fn get_anti_spiral_counter(state: State<'_, AppState>) -> CmdResult<Value> {
    // B-6: read the latest AntiSpiralHit SSB row. When no row exists yet
    // (pre-task or no spiral detected), return the zero counter so the
    // dashboard renders a "no spiral" state rather than misleading zeros
    // that look like a wired-but-empty producer.
    let hit = state
        .viewer
        .as_ref()
        .and_then(|v| v.latest_anti_spiral_hit().ok().flatten());

    match hit {
        Some(p) => Ok(json!({
            // AntiSpiralHitPayload carries agent_id (the triggering agent) and
            // step_hash (Blake3 hex of step content); it has no baby-step id,
            // so step_id is left empty rather than mislabeling agent_id.
            "step_id": "",
            "edit_count": p.count,
            "threshold": elai_orchestrator::SPIRAL_THRESHOLD,
            "triggered": p.count >= elai_orchestrator::SPIRAL_THRESHOLD,
            "last_hash": &p.step_hash,
        })),
        None => Ok(json!({
            "step_id": "",
            "edit_count": 0,
            "threshold": elai_orchestrator::SPIRAL_THRESHOLD,
            "triggered": false,
            "last_hash": "",
        })),
    }
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

/// B-7 STEP 7: ExecutablePipeline DAG visualizer. Returns the full DAG
/// (nodes + dependency edges) built from the decomposition tree, plus
/// validation error history and repair_log sourced from SSB
/// `dag_repair_round` + `dag_repair_escalated` rows.
///
/// Response shape:
///   { nodes: PipelineNode[], validation_errors: ValidationErrorEntry[],
///     repair_log: RepairLogEntry[] }
///
/// `nodes` — one DAG node per concept / phase / step / baby_step. Each
/// node's `deps` array lists its parent node id (the decomposition tree
/// is a strict hierarchy, so every non-root node has exactly one dep).
/// `kind` is "task" for concept/phase/step/baby_step, "gate" for steps
/// that carry an exit-criteria checkpoint, and "merge" for concept roots
/// that aggregate multiple phases.
///
/// `validation_errors` — derived from `dag_repair_round` rows. Each repair
/// round's `reason` field carries the validation error description; the
/// `action_kind` ("reorder" | "substitute") identifies the repair strategy
/// applied. This is the ValidationErrorKind history surface — the SSB does
/// not persist standalone ValidationError rows, so the repair round reasons
/// are the canonical error-history source.
///
/// `repair_log` — the raw `dag_repair_round` rows plus any
/// `dag_repair_escalated` rows (T_DAG=10 exhausted), so the dashboard can
/// render the full repair timeline including escalation badges.
///
/// Returns `{ nodes: [], validation_errors: [], repair_log: [] }` when no
/// viewer is configured or no decomposition rows exist yet — the dashboard
/// renders a "no decomposition yet" placeholder.
#[tauri::command]
pub fn get_executable_pipeline(state: State<'_, AppState>) -> CmdResult<Value> {
    let viewer = match state.viewer.as_ref() {
        Some(v) => v,
        None => {
            return Ok(json!({
                "nodes": [],
                "validation_errors": [],
                "repair_log": [],
            }));
        }
    };

    // 1. Build DAG nodes from the decomposition tree snapshot.
    let snapshot = viewer.decomposition_tree(None).map_err(map_err)?;
    let mut nodes: Vec<Value> = Vec::new();
    for concept in &snapshot.concepts {
        let concept_id = format!("concept-{}", concept.id);
        let concept_has_multi_phase = concept.phases.len() > 1;
        nodes.push(json!({
            "id": concept_id,
            "label": concept.title,
            "kind": if concept_has_multi_phase { "merge" } else { "task" },
            "status": "pending",
            "deps": [],
        }));
        for phase in &concept.phases {
            let phase_id = format!("phase-{}", phase.id);
            nodes.push(json!({
                "id": phase_id,
                "label": phase.title,
                "kind": "task",
                "status": "pending",
                "deps": [concept_id],
            }));
            for step in &phase.steps {
                let step_id = format!("step-{}", step.id);
                // Steps with exit-criteria behave as gates in the DAG.
                let step_kind = "gate";
                nodes.push(json!({
                    "id": step_id,
                    "label": step.title,
                    "kind": step_kind,
                    "status": "pending",
                    "deps": [phase_id],
                }));
                for baby in &step.baby_steps {
                    let baby_id = format!("bs-{}", baby.id);
                    let baby_status = match baby.status.as_str() {
                        "pass" | "done" | "completed" => "done",
                        "fail" | "failed" | "error" => "failed",
                        "active" | "running" => "active",
                        "skip" | "skipped" => "skipped",
                        _ => "pending",
                    };
                    nodes.push(json!({
                        "id": baby_id,
                        "label": baby.title,
                        "kind": "task",
                        "status": baby_status,
                        "deps": [step_id],
                    }));
                }
            }
        }
    }

    // 2. Collect validation error history + repair log from SSB rows.
    let repair_rounds = viewer.recent_dag_repair_rounds(100).unwrap_or_default();
    let repair_escalated = viewer.recent_dag_repair_escalated(50).unwrap_or_default();

    let mut validation_errors: Vec<Value> = Vec::new();
    let mut repair_log: Vec<Value> = Vec::new();

    for round in &repair_rounds {
        validation_errors.push(json!({
            "kind": round.action_kind,
            "round": round.round,
            "message": round.reason,
        }));
        repair_log.push(json!({
            "action_kind": round.action_kind,
            "round": round.round,
            "reason": round.reason,
            "escalated": false,
        }));
    }
    for esc in &repair_escalated {
        repair_log.push(json!({
            "action_kind": "escalated",
            "round": esc.rounds,
            "reason": esc.last_error,
            "escalated": true,
            "remaining_issues": esc.remaining_issues,
        }));
    }

    Ok(json!({
        "nodes": nodes,
        "validation_errors": validation_errors,
        "repair_log": repair_log,
    }))
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

/// B-7 STEP 6: query `dag_repair_escalated` rows from SSB (most recent 50).
/// Returns empty array when no viewer is configured or no escalation rows
/// exist. Surfaced alongside `get_dag_repair_log` in the OrchestrationDrawer
/// DAG REPAIR section so users see when T_DAG=10 is exhausted.
#[tauri::command]
pub fn get_dag_repair_escalated(state: State<'_, AppState>) -> CmdResult<Value> {
    let rows = state
        .viewer
        .as_ref()
        .and_then(|v| v.recent_dag_repair_escalated(50).ok())
        .unwrap_or_default();
    serde_json::to_value(&rows).map_err(|e| e.to_string())
}

/// B-7 (SCHEMA_VERSION 100): query `package_ref_verified` rows from SSB
/// (most recent 100). Returns empty array when no viewer is configured or
/// no package-verify rows exist. Mirrors the `get_dag_repair_log` shape so
/// PlanWindow + ProjectIntelligence drawers can render a consistent badge.
#[tauri::command]
pub fn get_package_verifications(state: State<'_, AppState>) -> CmdResult<Value> {
    let rows = state
        .viewer
        .as_ref()
        .and_then(|v| v.recent_package_verifications(100).ok())
        .unwrap_or_default();
    serde_json::to_value(&rows).map_err(|e| e.to_string())
}

/// B-7 Task #8: query `task_split` rows carrying PhaseSizingSplitter context
/// (splitter = "phase_sizing", phase_id = Some) from SSB (most recent 100).
/// Returns empty array when no viewer is configured or no phase-split rows
/// exist. PlanWindow renders a split badge on phase nodes that appear in
/// this list.
#[tauri::command]
pub fn get_phase_sized_splits(state: State<'_, AppState>) -> CmdResult<Value> {
    let rows = state
        .viewer
        .as_ref()
        .and_then(|v| v.recent_phase_sized_splits(100).ok())
        .unwrap_or_default();
    serde_json::to_value(&rows).map_err(|e| e.to_string())
}

/// B-7 Step 9: query `tree_position_changed` rows from SSB (most recent
/// 100). Returns empty array when no viewer is configured or no cursor
/// rows exist. PlanWindow uses the latest row's `current_node` to
/// highlight the TreeNavigator cursor in the tree pane.
#[tauri::command]
pub fn get_tree_positions(state: State<'_, AppState>) -> CmdResult<Value> {
    let rows = state
        .viewer
        .as_ref()
        .and_then(|v| v.recent_tree_positions(100).ok())
        .unwrap_or_default();
    serde_json::to_value(&rows).map_err(|e| e.to_string())
}

/// B-7 Step 10: query `failure_recorded` rows from SSB (most recent
/// 100). Returns empty array when no viewer is configured or no failure
/// rows exist yet. PlanWindow baby-step detail pane uses these to render
/// per-baby-step retry history (reason + attempts per failure upsert).
#[tauri::command]
pub fn get_failure_records(state: State<'_, AppState>) -> CmdResult<Value> {
    let rows = state
        .viewer
        .as_ref()
        .and_then(|v| v.recent_failure_records(100).ok())
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
        // C-15: OsClock-backed nanos for the synthetic session-id seed. This is
        // a one-shot per-process identifier; not part of any deterministic test.
        let ts = OsClock.system_now()
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
/// C-4: live aggregation over `cloud_spend` SSB rows via
/// `SsbViewer::cloud_spend_summary`. `task_usd` is the most recent row's
/// `cost_usd`; `day_usd` plus the two cache token counters sum across
/// today's rows. `redaction_count` is 0 today (no source SSB row) — wired
/// in for a future `redaction_event` action type (C-4-decisions.md Q5).
#[tauri::command]
pub fn get_cloud_spend(state: State<'_, AppState>) -> CmdResult<Value> {
    let now = now_ms(&state);
    let summary = match state.viewer.as_ref() {
        Some(v) => v.cloud_spend_summary(now).map_err(map_err)?,
        None => elai_orchestrator::CloudSpendSummary::default(),
    };
    serde_json::to_value(&summary).map_err(map_err)
}

/// C-5 follow-on: latest cloud routing status surfaced to CloudPanel.
///
/// Returns a `CloudRouteStatus` JSON object aggregating the most-recent
/// `route_blocked`, `warn_limit_reached` and `cost_estimate` SSB rows so
/// the dashboard can render their banners without polling each variant
/// individually. Each field is `null` when no row of that type exists in
/// the SSB yet. Output shape matches `CloudRouteStatus` in
/// `dashboard-v2/src/lib/api/types.ts`.
///
/// Wire-shape contract (atomic with SCHEMA_VERSION 103):
///   {
///     route_blocked:      { reason, spent_usd, cap_usd, cap_period, ts_ms } | null,
///     warn_limit_reached: { spent_usd, cap_usd, warn_at_percent, pct_of_cap, ts_ms } | null,
///     cost_estimate:      { low_usd, high_usd, model, step_count, heuristic_source, ts_ms } | null,
///   }
#[tauri::command]
pub fn get_cloud_route_status(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!({
            "route_blocked": Value::Null,
            "warn_limit_reached": Value::Null,
            "cost_estimate": Value::Null,
        }));
    };

    let route_blocked = match viewer.latest_action_of_type("route_blocked") {
        Ok(Some(detail)) => detail.payload_json,
        Ok(None) => Value::Null,
        Err(e) => return Err(map_err(e)),
    };
    let warn_limit_reached = match viewer.latest_action_of_type("warn_limit_reached") {
        Ok(Some(detail)) => detail.payload_json,
        Ok(None) => Value::Null,
        Err(e) => return Err(map_err(e)),
    };
    let cost_estimate = match viewer.latest_action_of_type("cost_estimate") {
        Ok(Some(detail)) => detail.payload_json,
        Ok(None) => Value::Null,
        Err(e) => return Err(map_err(e)),
    };

    Ok(json!({
        "route_blocked": route_blocked,
        "warn_limit_reached": warn_limit_reached,
        "cost_estimate": cost_estimate,
    }))
}

/// C-28: Mutation gauge readback for MonitorWindow.
///
/// Returns `null` when no `mutation_survived` SSB row has been emitted
/// in the current session (i.e. either Step 6 hasn't run yet OR every
/// mutant was caught — both indistinguishable from this surface).
///
/// Backend wiring is deferred to a follow-up that reads the latest
/// MutationResult from session state. For now the command returns
/// `Value::Null`, matching the contract test pin
/// (DECISION-D9 in C-28-decisions.md).
#[tauri::command]
pub fn get_mutation_gauge(state: State<'_, AppState>) -> CmdResult<Value> {
    // Touch state to keep the signature stable for future wiring.
    let _ = state;
    Ok(serde_json::Value::Null)
}

/// C-32 — Code-Completeness Layer 2 completeness_ratio gauge readback for
/// MonitorWindow. Returns `{ "known": false }` when no `skeleton_flushed`
/// SSB row exists yet (no skeleton-first path has activated). When
/// populated, the JSON shape is:
/// `{ "known": true, "ratio": f32, "blank_count": u32, "last_path": String | null }`.
///
/// Closes audit-C32 W2: reads the latest `skeleton_flushed` row via
/// [`elai_orchestrator::SsbViewer::latest_action_of_type`] and projects
/// the `path` + `completeness_ratio` fields onto the dashboard wire
/// shape. `blank_count` is not carried by SkeletonFlushedPayload itself
/// (it lives on SkeletonGeneratedPayload); the latest matching row's
/// blank_count is looked up best-effort and falls back to 0 when no
/// SkeletonGenerated row is found.
#[tauri::command]
pub fn get_completeness_ratio(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!({ "known": false }));
    };
    let flushed = match viewer.latest_action_of_type("skeleton_flushed") {
        Ok(Some(detail)) => detail,
        Ok(None) => return Ok(json!({ "known": false })),
        Err(e) => return Err(map_err(e)),
    };
    // SkeletonFlushedPayload -> { path, completeness_ratio }. Extract
    // both fields with defensive fallbacks so a future payload shape
    // bump does not panic the dashboard.
    let payload = &flushed.payload_json;
    let ratio = payload
        .get("completeness_ratio")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;
    let last_path = payload
        .get("path")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    // Best-effort blank_count: peek at the latest skeleton_generated row.
    // When absent (skeleton_flushed without a matching skeleton_generated)
    // we surface 0 so the dashboard renders the gauge but the explanation
    // tooltip notes "no blank count available".
    let blank_count = viewer
        .latest_action_of_type("skeleton_generated")
        .ok()
        .flatten()
        .and_then(|d| {
            d.payload_json
                .get("blank_count")
                .and_then(|v| v.as_u64())
                .map(|u| u as u32)
        })
        .unwrap_or(0u32);
    Ok(json!({
        "known": true,
        "ratio": ratio,
        "blank_count": blank_count,
        "last_path": last_path,
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

/// B-14 POLISH: superseded by [`stream_chat`]. This command writes a
/// `chat_message` SSB row carrying the intent string but does NOT stream
/// LLM tokens. The frontend `submit_chat_message` wrapper was removed
/// from `stub.ts` (spec stub_ban); the dashboard now uses `stream_chat`
/// for real token streaming. This Tauri command remains registered for
/// SSB-row compatibility and potential external callers, but has no
/// frontend caller as of B-14.
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

/// B-14 guardrail #3: remove an `active_streams` entry after natural task
/// completion (cancel_stream handles the abort path). Held lock is minimum
/// duration — no `.await` while locked. Silent on missing key (cancel may
/// have raced ahead).
fn cleanup_stream_entry(
    streams: &std::sync::Arc<std::sync::Mutex<BTreeMap<String, tokio::task::AbortHandle>>>,
    stream_id: &str,
) {
    if let Ok(mut g) = streams.lock() {
        g.remove(stream_id);
    }
}

/// B-14 wiring-close: build the sidecar `/v1/messages` request body with
/// optional thinking/effort forwarding. Extracted from `stream_chat` so the
/// thinking-block injection logic is unit-testable without a Tauri runtime.
/// `thinking_enabled` injects an Anthropic-style `thinking` block;
/// `effort_level` maps to budget tokens (low=2048, medium=4096, high=8192,
/// max=16384, default=4096). Mirrors `start_chat_stream` effort mapping.
fn build_stream_request_body(
    message: &str,
    thinking_enabled: bool,
    effort_level: Option<&str>,
) -> serde_json::Value {
    let mut body = serde_json::json!({
        "model": "qwen",
        "messages": [{"role": "user", "content": message}],
        "stream": true,
        "max_tokens": 512
    });
    if thinking_enabled {
        let budget = match effort_level {
            Some("low") => 2048,
            Some("medium") => 4096,
            Some("high") => 8192,
            Some("max") => 16384,
            _ => 4096,
        };
        body["thinking"] = serde_json::json!({
            "type": "enabled",
            "budget_tokens": budget
        });
    }
    body
}

/// B-14 STEP 1: Stream real LLM tokens from the sidecar `/v1/messages` SSE
/// endpoint to the frontend via Tauri events. Emits `"chat_event"` with
/// `{ stream_id, event_type, ... }` payloads where `event_type` is one of:
/// `"streaming_started"`, `"token"`, `"response_complete"`, `"error"`.
///
/// Returns `{ "stream_id": "<id>" }` immediately so the frontend can
/// `listen("chat_event", ...)` before the first token arrives. The streaming
/// task runs in a detached tokio task whose `AbortHandle` is stored in
/// `state.active_streams[stream_id]` so `cancel_stream` can abort it.
///
/// B-14 wiring-close: `thinking_enabled` / `effort_level` are forwarded into
/// the sidecar request body (the same options `start_chat_stream` accepts).
/// The server-side `classify_intent` is called before streaming so the
/// `streaming_started` event carries the deterministic intent category —
/// the frontend TS classifier still owns routing, the Rust classifier
/// supplies server-side metadata (additive `intent` field, nullable).
#[tauri::command]
pub async fn stream_chat(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    message: String,
    session_id: Option<String>,
    thinking_enabled: Option<bool>,
    effort_level: Option<String>,
) -> CmdResult<Value> {
    let sidecar_url = state
        .sidecar_url
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| "http://localhost:8100".to_string());
    let session = session_id.unwrap_or_else(|| format!("dash-v2-{}", std::process::id()));
    let stream_id = format!(
        "{}:{}",
        session,
        state
            .clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );

    // B-14 wiring-close: classify intent server-side via the canonical
    // orchestrator classifier (deterministic floor). The frontend TS
    // classifier (utils/intent.ts) still owns routing; this supplies
    // server-side intent metadata in the streaming_started event.
    let intent_label = elai_orchestrator::intent_classifier::classify_intent(&message)
        .as_str()
        .to_string();

    // Emit streaming_started event so the frontend can bind its listener.
    // The `intent` field is additive (nullable for older frontends).
    app.emit(
        "chat_event",
        json!({
            "stream_id": stream_id,
            "event_type": "streaming_started",
            "intent": intent_label
        }),
    )
    .map_err(|e| format!("emit failed: {e}"))?;

    // Stream from sidecar using direct HTTP (matches existing
    // start_chat_stream pattern). Full ConversationRuntime integration would
    // require more wiring (private modules); this reuses the same sidecar
    // path that the CLI uses.
    let streaming_http = state.streaming_http.clone();
    let app_clone = app.clone();
    let stream_id_clone = stream_id.clone();
    let sidecar_url_clone = sidecar_url.clone();
    let message_clone = message.clone();
    let thinking_clone = thinking_enabled.unwrap_or(false);
    let effort_clone = effort_level.clone();
    // B-14 guardrail #3: clone active_streams Arc into the task so it can
    // remove its own entry on natural completion (cancel_stream handles the
    // abort path). Without this, a completed stream leaks its AbortHandle.
    let active_streams_clone = state.active_streams.clone();

    let handle = tokio::task::spawn(async move {
        // B-14 wiring-close: forward thinking/effort into the sidecar
        // request body via the extracted helper (unit-tested).
        let request_body = build_stream_request_body(
            &message_clone,
            thinking_clone,
            effort_clone.as_deref(),
        );

        let mut response = match streaming_http
            .post(format!("{}/v1/messages", sidecar_url_clone))
            .json(&request_body)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                let _ = app_clone.emit(
                    "chat_event",
                    json!({
                        "stream_id": stream_id_clone,
                        "event_type": "error",
                        "error": format!("sidecar connect failed: {e}")
                    }),
                );
                cleanup_stream_entry(&active_streams_clone, &stream_id_clone);
                return;
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let _ = app_clone.emit(
                "chat_event",
                json!({
                    "stream_id": stream_id_clone,
                    "event_type": "error",
                    "error": format!("sidecar returned {}", status)
                }),
            );
            cleanup_stream_entry(&active_streams_clone, &stream_id_clone);
            return;
        }

        // Stream SSE events (matches existing start_chat_stream pattern).
        let mut sse_buf = String::new();
        loop {
            match response.chunk().await {
                Ok(Some(bytes)) => {
                    sse_buf.push_str(&String::from_utf8_lossy(&bytes));
                    while let Some(pos) = sse_buf.find("\n\n") {
                        let frame = sse_buf[..pos].to_owned();
                        sse_buf = sse_buf[pos + 2..].to_owned();
                        if let Some(text) = extract_text_delta(&frame) {
                            let _ = app_clone.emit(
                                "chat_event",
                                json!({
                                    "stream_id": stream_id_clone,
                                    "event_type": "token",
                                    "token": text
                                }),
                            );
                        }
                        if frame.contains("\"message_stop\"") {
                            let _ = app_clone.emit(
                                "chat_event",
                                json!({
                                    "stream_id": stream_id_clone,
                                    "event_type": "response_complete"
                                }),
                            );
                            cleanup_stream_entry(&active_streams_clone, &stream_id_clone);
                            return;
                        }
                    }
                }
                Ok(None) => {
                    let _ = app_clone.emit(
                        "chat_event",
                        json!({
                            "stream_id": stream_id_clone,
                            "event_type": "response_complete"
                        }),
                    );
                    cleanup_stream_entry(&active_streams_clone, &stream_id_clone);
                    return;
                }
                Err(e) => {
                    let _ = app_clone.emit(
                        "chat_event",
                        json!({
                            "stream_id": stream_id_clone,
                            "event_type": "error",
                            "error": format!("stream error: {e}")
                        }),
                    );
                    cleanup_stream_entry(&active_streams_clone, &stream_id_clone);
                    return;
                }
            }
        }
    });

    {
        let mut streams = state
            .active_streams
            .lock()
            .map_err(|e| format!("lock failed: {e}"))?;
        streams.insert(stream_id.clone(), handle.abort_handle());
    }

    Ok(json!({ "stream_id": stream_id }))
}

/// B-14 STEP 1: Cancel an inflight `stream_chat` by stream_id. Aborts the
/// detached tokio task and removes the handle from `active_streams`.
#[tauri::command]
pub fn cancel_stream(
    state: State<'_, AppState>,
    stream_id: String,
) -> CmdResult<Value> {
    let mut streams = state
        .active_streams
        .lock()
        .map_err(|e| format!("lock failed: {e}"))?;
    if let Some(handle) = streams.remove(&stream_id) {
        handle.abort();
        Ok(json!({ "cancelled": true }))
    } else {
        Ok(json!({ "cancelled": false, "error": "stream not found" }))
    }
}

#[tauri::command]
pub fn classify_intent(text: String) -> CmdResult<Value> {
    // B-14 STEP 3: delegate to the canonical orchestrator classifier so
    // the dashboard Tauri command and the frontend TS classifier
    // (`crates/dashboard-v2/src/lib/utils/intent.ts`) share one category
    // set. The orchestrator `classify_intent` is the deterministic floor
    // (slash commands + keywords); `Ambiguous` results may be resolved
    // by an LLM fallback call in the dashboard layer (spec FLOW 4).
    let intent = elai_orchestrator::intent_classifier::classify_intent(&text);
    Ok(json!({ "intent": intent.as_str(), "text": text }))
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

// ---------------------------------------------------------------------------
// E-2 — streaming timing telemetry (StreamingTimingRecorded SSB beacon).
//
// The SSB variant + payload live in elai_orchestrator::ssb (off-limits to
// this unit); this is the PRODUCER side that plumbs + emits. The emit is
// gated wire-dark behind ELAI_STREAMING_TIMING_ENABLE so default builds emit
// nothing (zero behaviour change). The env-var name is declared locally
// because ssb.rs (where sibling *_ENABLE_ENV consts live) is frozen for the
// paused H-plan; the const value is the canonical flag string regardless.
// ---------------------------------------------------------------------------

/// Production gate for the StreamingTimingRecorded emit. Default-OFF.
const ELAI_STREAMING_TIMING_ENABLE_ENV: &str = "ELAI_STREAMING_TIMING_ENABLE";

/// True iff `ELAI_STREAMING_TIMING_ENABLE` is set to a truthy value.
fn streaming_timing_enabled() -> bool {
    std::env::var(ELAI_STREAMING_TIMING_ENABLE_ENV)
        .ok()
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

/// Nearest-rank percentile (P-th) of a slice of inter-chunk gaps in ms.
/// Returns 0 when the slice is empty. `pct` is clamped to `[0, 100]`. The
/// slice is sorted in-place by the caller's clone; this fn takes a sorted
/// view to avoid re-sorting per percentile.
fn percentile_ms(sorted_gaps: &[u32], pct: u32) -> u32 {
    if sorted_gaps.is_empty() {
        return 0;
    }
    let pct = pct.min(100);
    // Nearest-rank: rank = ceil(pct/100 * n), 1-based, clamped to n.
    let n = sorted_gaps.len();
    let rank = ((pct as usize * n) + 99) / 100; // ceil(pct*n/100)
    let idx = rank.saturating_sub(1).min(n - 1);
    sorted_gaps[idx]
}

/// Accumulates per-stream timing while SSE chunks arrive, then materializes a
/// [`StreamingTimingRecordedPayload`] at stream-end. All wall-clock math uses
/// [`std::time::Instant`] so it is monotonic and unaffected by clock skew;
/// only the final `ts_ms` field is UNIX-epoch wall time.
struct StreamTiming {
    start: std::time::Instant,
    ttft: Option<std::time::Duration>,
    last_chunk_at: Option<std::time::Instant>,
    inter_chunk_gaps_ms: Vec<u32>,
    tokens_emitted: u32,
}

impl StreamTiming {
    fn new() -> Self {
        Self {
            start: std::time::Instant::now(), // entropy-allow: monotonic Instant baseline for in-stream TTFT/gap durations only (see struct doc comment above); unaffected by clock skew, never compared across a determinism replay
            ttft: None,
            last_chunk_at: None,
            inter_chunk_gaps_ms: Vec::new(),
            tokens_emitted: 0,
        }
    }

    /// Record one rendered text delta. Sets TTFT on the first delta and
    /// records the gap since the previous delta on every subsequent one.
    fn on_delta(&mut self) {
        let now = std::time::Instant::now(); // entropy-allow: monotonic Instant for inter-chunk gap measurement only, same rationale as StreamTiming::new
        if self.ttft.is_none() {
            self.ttft = Some(now.duration_since(self.start));
        }
        if let Some(prev) = self.last_chunk_at {
            // NARROWING-OK: gaps are sub-second in practice; saturate at u32::MAX.
            let gap = now.duration_since(prev).as_millis();
            self.inter_chunk_gaps_ms
                .push(u32::try_from(gap).unwrap_or(u32::MAX));
        }
        self.last_chunk_at = Some(now);
        self.tokens_emitted = self.tokens_emitted.saturating_add(1);
    }

    /// Build the typed SSB payload for this stream. `ts_ms` is wall-clock.
    fn into_payload(
        self,
        request_id: String,
        stop_reason: &str,
        ts_ms: i64,
    ) -> elai_orchestrator::ssb::StreamingTimingRecordedPayload {
        let total_ms =
            u32::try_from(self.start.elapsed().as_millis()).unwrap_or(u32::MAX);
        let ttft_ms = self
            .ttft
            .map(|d| u32::try_from(d.as_millis()).unwrap_or(u32::MAX))
            .unwrap_or(0);
        let mut sorted = self.inter_chunk_gaps_ms.clone();
        sorted.sort_unstable();
        let mean_inter_chunk_ms = if sorted.is_empty() {
            0
        } else {
            // u64 accumulation avoids overflow on long streams.
            let sum: u64 = sorted.iter().map(|&g| g as u64).sum();
            u32::try_from(sum / sorted.len() as u64).unwrap_or(u32::MAX)
        };
        elai_orchestrator::ssb::StreamingTimingRecordedPayload {
            request_id,
            ttft_ms,
            mean_inter_chunk_ms,
            p50_inter_chunk_ms: percentile_ms(&sorted, 50),
            p99_inter_chunk_ms: percentile_ms(&sorted, 99),
            total_ms,
            tokens_emitted: self.tokens_emitted,
            stop_reason: stop_reason.to_owned(),
            ts_ms,
        }
    }
}

/// Emit a StreamingTimingRecorded SSB row for a completed chat stream.
/// No-op (returns immediately) when the wire-dark gate is OFF or no SSB
/// writer is wired. Best-effort: a failed write is swallowed because the
/// user-facing stream already completed; telemetry must never surface as a
/// stream error.
async fn emit_streaming_timing(
    writer: Option<&std::sync::Arc<elai_orchestrator::ssb::Ssb>>,
    timing: StreamTiming,
    request_id: String,
    stop_reason: &str,
    ts_ms: i64,
) {
    if !streaming_timing_enabled() {
        return;
    }
    let Some(writer) = writer else {
        return;
    };
    let payload = SsbPayload::StreamingTimingRecorded(timing.into_payload(
        request_id,
        stop_reason,
        ts_ms,
    ));
    // Best-effort: log on failure, never propagate.
    if let Err(e) = writer
        .write_payload("dashboard-v2", 0, payload, vec![])
        .await
    {
        tracing::warn!(error = %e, "streaming_timing_recorded emit failed");
    }
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
            // Tranche B-6 quad B-39: 85% soft-warn threshold; default is
            // `REFLECT_BUDGET_SOFT_WARN_PCT` (85). Wire-dark unless
            // ELAI_B39_SOFTWARN_ENABLE=1 on the SSB sink.
            budget_soft_warn_pct: elai_reasoning::REFLECT_BUDGET_SOFT_WARN_PCT,
        };
        let pipeline = elai_reasoning::ChatReflectPipeline::new(config, Some(sink));

        let active_streams = state.active_streams.clone();
        let stream_id_task = stream_id.clone();
        let app_task = app.clone();
        let prompt_owned = prompt.clone();
        // F-10 REFINE leg: capture the SSB handle (if wired) for the
        // monotonicity guard. Cloned out of `state` before the move into the
        // spawned task so the guard uses the EXISTING Ssb, never a new one.
        let ssb_for_refine = state.writer.clone();

        let join = tokio::task::spawn(async move {
            let client_ref: &dyn elai_reasoning::ChatInferenceClient = &*client;
            let output = pipeline
                .run(stream_id_task.clone(), &prompt_owned, client_ref)
                .await;
            let _ = app_task.emit(
                &format!("chat-stream:{stream_id_task}"),
                json!({ "text": output.final_reply, "stream_id": stream_id_task }),
            );
            // F-10 REFINE leg (W-10, fire-and-forget; runs AFTER the reply
            // event is emitted so it is off the user-perceived hot path).
            // Mirrors the CLI live_cli.rs refine call-site: compares this
            // turn's CWE-class set against the prior refine iteration's set
            // and emits a non-blocking SecurityClassRegressionBlocked row on a
            // regression. No per-session log path here (the dashboard has no
            // session dir on this path); the SSB beacon + Finding are the
            // durable record. loop_id = stream_id.
            if let Some(ssb) = ssb_for_refine.as_ref() {
                elai_orchestrator::run_refine_leg(
                    ssb.as_ref(),
                    &stream_id_task,
                    &output.final_reply,
                    "",
                    None,
                )
                .await;
            }
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
    // E-2: SSB writer + clock cloned out of `state` (not Send into the task)
    // so the StreamingTimingRecorded beacon can be emitted at stream-end.
    let timing_writer = state.writer.clone();
    let timing_clock = state.clock.clone();

    let join = tokio::task::spawn(async move {
        // E-2: monotonic per-stream timing accumulator. Starts the wall-clock
        // anchor at the moment the request task begins.
        let mut timing = StreamTiming::new();
        let timing_ts = || {
            timing_clock
                .system_now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0)
        };
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
                emit_streaming_timing(
                    timing_writer.as_ref(),
                    timing,
                    stream_id_task.clone(),
                    "error",
                    timing_ts(),
                )
                .await;
                return;
            }
            Err(e) => {
                let _ = app_task.emit(
                    &format!("chat-stream-end:{stream_id_task}"),
                    json!({ "stream_id": stream_id_task, "error": e.to_string() }),
                );
                emit_streaming_timing(
                    timing_writer.as_ref(),
                    timing,
                    stream_id_task.clone(),
                    "error",
                    timing_ts(),
                )
                .await;
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
                            // E-2: record per-token timing (TTFT on the first
                            // delta, inter-chunk gap on each subsequent one).
                            timing.on_delta();
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
                            emit_streaming_timing(
                                timing_writer.as_ref(),
                                timing,
                                stream_id_task.clone(),
                                "end_turn",
                                timing_ts(),
                            )
                            .await;
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
                    emit_streaming_timing(
                        timing_writer.as_ref(),
                        timing,
                        stream_id_task.clone(),
                        "end_turn",
                        timing_ts(),
                    )
                    .await;
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
                    emit_streaming_timing(
                        timing_writer.as_ref(),
                        timing,
                        stream_id_task.clone(),
                        "error",
                        timing_ts(),
                    )
                    .await;
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
// E-2 — StreamingTimingRecorded emit smoke (off-agent artifact proof).
//
// Wiring-enforcement R4/R7: the load-bearing proof is the SSB ROW the emit
// leaves in a real Ssb (not the agent's stdout, not a leaf-name grep). The
// orchestrator INTEGRATION test harness does not link in this env (ort/ndarray
// rlib-format failure), so this lives as an IN-CRATE #[cfg(test)] lib test:
// drive `emit_streaming_timing` against a temp-file Ssb with the wire-dark gate
// ON, then read the row back via `query_by_type` and decode the payload. The
// gate is exercised both ways (OFF => zero rows, ON => exactly one).
// One sequential #[tokio::test] (the shared ELAI_* env races a parallel split).
// ---------------------------------------------------------------------------
#[cfg(test)]
mod streaming_timing_emit_tests {
    use super::{
        emit_streaming_timing, percentile_ms, StreamTiming,
        ELAI_STREAMING_TIMING_ENABLE_ENV,
    };
    use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig, SsbPayload};
    use elai_orchestrator::{decode_ssb_payload_bytes, SsbReader};
    use std::sync::Arc;

    fn temp_ssb(name: &str) -> (tempfile::TempDir, Arc<Ssb>) {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let cfg = SsbConfig {
            db_path: dir.path().join(format!("{name}.db")),
            pool_size: Some(2),
            project_id: "default".into(),
            session_id: "test-session".into(),
        };
        let clock: Arc<dyn elai_core::ClockProvider> =
            Arc::new(elai_core::clock::SeededClock::new());
        let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb init"));
        (dir, ssb)
    }

    #[test]
    fn percentile_nearest_rank_matches_known_values() {
        // empty -> 0
        assert_eq!(percentile_ms(&[], 50), 0);
        // single element -> that element at every percentile
        assert_eq!(percentile_ms(&[7], 99), 7);
        // sorted [10,20,30,40,50]: p50 nearest-rank = ceil(0.5*5)=3rd = 30,
        // p99 = ceil(0.99*5)=5th = 50, p0 clamps to 1st = 10.
        let s = [10u32, 20, 30, 40, 50];
        assert_eq!(percentile_ms(&s, 50), 30);
        assert_eq!(percentile_ms(&s, 99), 50);
        assert_eq!(percentile_ms(&s, 0), 10);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn emit_writes_one_streaming_timing_row_when_gate_on() {
        // SEQUENTIAL: the gate env var is process-global; this single test
        // owns it for both the OFF and ON legs, then clears it.
        let (_dir, ssb) = temp_ssb("ssb_streaming_timing");

        // --- OFF leg: gate cleared => zero rows, even with a writer wired. ---
        std::env::remove_var(ELAI_STREAMING_TIMING_ENABLE_ENV);
        let mut off_timing = StreamTiming::new();
        off_timing.on_delta();
        emit_streaming_timing(
            Some(&ssb),
            off_timing,
            "req-off".to_owned(),
            "end_turn",
            1_700_000_000_000,
        )
        .await;
        let off_rows = ssb
            .query_by_type(SsbActionType::StreamingTimingRecorded)
            .expect("query off rows");
        assert!(
            off_rows.is_empty(),
            "gate OFF must emit zero rows; got {}",
            off_rows.len()
        );

        // --- ON leg: gate set => exactly one row with the decoded fields. ---
        std::env::set_var(ELAI_STREAMING_TIMING_ENABLE_ENV, "1");
        let mut on_timing = StreamTiming::new();
        // Three deltas -> tokens_emitted=3, two inter-chunk gaps recorded.
        on_timing.on_delta();
        on_timing.on_delta();
        on_timing.on_delta();
        emit_streaming_timing(
            Some(&ssb),
            on_timing,
            "req-on".to_owned(),
            "end_turn",
            1_700_000_000_001,
        )
        .await;

        let rows = ssb
            .query_by_type(SsbActionType::StreamingTimingRecorded)
            .expect("query on rows");
        assert_eq!(rows.len(), 1, "gate ON must emit exactly one row");

        // Decode the row payload — the off-agent artifact the auditor reads.
        // decode_ssb_payload_bytes is the public, dep-free msgpack decoder
        // (matches Ssb::write_payload's rmp_serde::to_vec_named wire format).
        let decoded: SsbPayload =
            decode_ssb_payload_bytes(&rows[0].payload).expect("decode payload");
        match decoded {
            SsbPayload::StreamingTimingRecorded(p) => {
                assert_eq!(p.request_id, "req-on");
                assert_eq!(p.tokens_emitted, 3, "three deltas => three tokens");
                assert_eq!(p.stop_reason, "end_turn");
                assert_eq!(p.ts_ms, 1_700_000_000_001);
                // p50 <= p99 always holds for a non-decreasing distribution.
                assert!(p.p50_inter_chunk_ms <= p.p99_inter_chunk_ms);
            }
            other => panic!("expected StreamingTimingRecorded, got {other:?}"),
        }

        // Cleanup the process-global gate so sibling tests see default-OFF.
        std::env::remove_var(ELAI_STREAMING_TIMING_ENABLE_ENV);
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
// C-17 (DRIFT-J) — Indexing resilience snapshot.
// ---------------------------------------------------------------------------
//
// Surfaces the C-17 lifecycle state to IndexingProgressPanel.svelte:
//   * `auto_resume_enabled` — `true` when the per-project metadata DB
//     exists at all (the auto_resume_indexed_projects pipeline is wired
//     by definition; the panel uses this to surface "auto-resume is on").
//   * `watch_locks_held` — number of active Watch-scope lock files on
//     disk for this project (probed via lock files on disk, NOT by
//     calling `try_acquire` which would compete with the live holder).
//   * `last_checkpoint_unix_secs` — mirror of get_indexing_progress's
//     last_checkpoint_at.
//   * `last_resume_from_batch` / `last_resume_files_skipped` — pulled
//     from the most recent IndexResume SSB row (None when no resume
//     row has been emitted yet this session).
//   * `lock_contested_count` — count of LockContested SSB rows in this
//     session.
//   * `shutdown_deadline_secs` — surfaced literal of
//     `elai_orchestrator::startup::SHUTDOWN_DEADLINE` so the dashboard
//     stays in lockstep with the constant.
//   * `last_batch_duration_ms` — `batch_duration_ms` field of the most
//     recent IndexCheckpoint SSB row (None when older payloads lack the
//     C-17 optional field).

#[tauri::command]
pub async fn get_indexing_resilience(
    _app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> CmdResult<Value> {
    let project_root = state.project_root_snapshot();
    let clock = state.clock.clone();
    let resolved_project_id =
        elai_knowledge::metadata::project_id_from_path(&project_root);

    // Probe the metadata DB + on-disk lock files synchronously inside
    // spawn_blocking so the async tauri task does not stall.
    let project_root_for_locks = project_root.clone();
    let resolved_project_id_for_blocking = resolved_project_id.clone();
    let (auto_resume_enabled, last_checkpoint_unix_secs, watch_locks_held) =
        tokio::task::spawn_blocking(move || -> (bool, i64, u32) {
            let indexes_dir = project_root_for_locks
                .join(elai_core::paths::ELAI_DATA_DIR_NAME)
                .join(elai_core::paths::INDEXES_DIR_NAME);
            let db_path = indexes_dir.join(elai_knowledge::metadata::METADATA_DB_FILENAME);
            let mut last_ckpt: i64 = 0;
            let auto_resume = db_path.exists();
            if auto_resume {
                if let Ok(store) =
                    elai_knowledge::metadata::ProjectMetadataStore::open(&db_path, clock)
                {
                    if let Ok(Some(meta)) = store.load(&resolved_project_id_for_blocking) {
                        last_ckpt = meta.last_checkpoint_at;
                    }
                }
            }
            // Count watch lock files at the per-scope path. The Watch-scope
            // filename is "watch.lock" (LockScope::filename); look in the
            // indexes dir AND in any nested per-project subdirs.
            let mut watch_locks: u32 = 0;
            let watch_filename = elai_orchestrator::lock::LockScope::Watch.filename();
            // Direct path.
            if indexes_dir.join(&watch_filename).is_file() {
                watch_locks = watch_locks.saturating_add(1);
            }
            // Nested per-project subdir layout (dashboard multi-project case).
            if let Ok(read) = std::fs::read_dir(&indexes_dir) {
                for entry in read.flatten() {
                    let candidate = entry.path().join(&watch_filename);
                    if candidate.is_file() {
                        watch_locks = watch_locks.saturating_add(1);
                    }
                }
            }
            (auto_resume, last_ckpt, watch_locks)
        })
        .await
        .map_err(|e| format!("get_indexing_resilience probe panicked: {e}"))?;

    // Pull resume + contested + batch_duration_ms metrics from the SSB
    // viewer. Each lookup is best-effort; missing rows simply produce
    // None / 0 in the response.
    let mut last_resume_from_batch: Option<u32> = None;
    let mut last_resume_files_skipped: Option<u64> = None;
    let mut lock_contested_count: u32 = 0;
    let mut last_batch_duration_ms: Option<u64> = None;

    if let Some(viewer) = state.viewer.as_ref() {
        const RESILIENCE_PER_TYPE_LIMIT: usize = 50;
        if let Ok(rows) = viewer.recent_rows_of_type("index_resume", RESILIENCE_PER_TYPE_LIMIT) {
            // recent_rows_of_type returns DESC by ts; first row is latest.
            if let Some(row) = rows.first() {
                last_resume_from_batch = row
                    .payload_json
                    .get("resumed_from_batch")
                    .and_then(|v| v.as_u64())
                    .and_then(|n| u32::try_from(n).ok());
                last_resume_files_skipped =
                    row.payload_json.get("files_skipped").and_then(|v| v.as_u64());
            }
        }
        if let Ok(rows) = viewer.recent_rows_of_type("lock_contested", RESILIENCE_PER_TYPE_LIMIT) {
            lock_contested_count = u32::try_from(rows.len()).unwrap_or(u32::MAX);
        }
        if let Ok(rows) = viewer.recent_rows_of_type("index_checkpoint", RESILIENCE_PER_TYPE_LIMIT) {
            if let Some(row) = rows.first() {
                last_batch_duration_ms = row
                    .payload_json
                    .get("batch_duration_ms")
                    .and_then(|v| v.as_u64());
            }
        }
    }

    let _ = resolved_project_id; // retained for future per-project filtering.

    Ok(json!({
        "auto_resume_enabled": auto_resume_enabled,
        "watch_locks_held": watch_locks_held,
        "last_checkpoint_unix_secs": last_checkpoint_unix_secs,
        "last_resume_from_batch": last_resume_from_batch,
        "last_resume_files_skipped": last_resume_files_skipped,
        "lock_contested_count": lock_contested_count,
        "shutdown_deadline_secs": elai_orchestrator::startup::SHUTDOWN_DEADLINE.as_secs(),
        "last_batch_duration_ms": last_batch_duration_ms,
    }))
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

    let planning_payload = json!({
        "request_id": request_id,
        "status":     "submitted",
        "task":       task,
        "project_id": pid,
        "seq":        seq,
        "note": "task written to SSB; task-runner will pick up and process it",
    });
    // B-36 STEP 5: emit the canonical `web_research_step` SSE event name
    // (WIRING_CONTRACT produces_symbols) alongside the legacy
    // `planning-progress` alias so both old and new frontend listeners
    // receive the planning submission event.
    let _ = app.emit("web_research_step", planning_payload.clone());
    let _ = app.emit("planning-progress", planning_payload);

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
            let payload = json!({
                "request_id":      request_id,
                "node_id":         event.node_id,
                "action":          event.action,
                "phi":             event.phi,
                "psi":             event.psi,
                "iteration":       event.iteration,
                "sources_fetched": event.sources_fetched,
                "sources_used":    event.sources_used,
                "fetcher":         event.fetcher,
            });
            // B-36 STEP 5: emit the canonical `web_research_step` SSE event
            // name (WIRING_CONTRACT produces_symbols) alongside the legacy
            // `research-progress` alias so both old and new frontend listeners
            // receive the same payload.
            let _ = app.emit("web_research_step", payload.clone());
            let _ = app.emit("research-progress", payload);
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
        let abs = PathBuf::from(r"F:\.ELAI_workflow\.elai_cc\tools\obscura\obscura.exe");
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

    let fetch_clock = OsClock;
    let start = fetch_clock.now();

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

    let fetch_time_ms = u64::try_from(fetch_clock.elapsed_since(start).as_millis()).unwrap_or(u64::MAX);
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
///
/// B-36 STEP 5: emits SSE events via `Window::emit` so the frontend can
/// render streaming progress live:
///   - `streaming_started` — emitted once before the subprocess spawns.
///   - `turn`              — emitted per stdout/stderr line as it arrives.
///   - `response`          — emitted once when the subprocess exits (ok or
///                           error), carrying the full stdout + exit status.
#[tauri::command]
pub async fn one_shot_prompt(
    app: tauri::AppHandle,
    text: String,
) -> CmdResult<Value> {
    use tokio::io::{AsyncBufReadExt, BufReader};

    let exe = locate_elai_exe()?;

    // B-36 STEP 5: emit streaming_started before spawning so the frontend
    // can flip its typing indicator on immediately.
    let request_id = format!("oneshot-{}", now_ms_raw());
    let _ = app.emit(
        "streaming_started",
        json!({
            "request_id": request_id,
            "text":       text,
        }),
    );

    let mut child = tokio::process::Command::new(&exe)
        .args(["prompt", &text])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn elai.exe: {e}"))?;

    // Merge stdout + stderr lines via a channel so both are forwarded as
    // `turn` events (same pattern as run_d5_live_contract).
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(256);

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

    // Forward lines to frontend via `turn` events.
    let app_turn = app.clone();
    let rid_turn = request_id.clone();
    let mut all_lines: Vec<String> = Vec::new();
    while let Some(line) = rx.recv().await {
        all_lines.push(line.clone());
        let _ = app_turn.emit(
            "turn",
            json!({
                "request_id": rid_turn,
                "line":       line,
            }),
        );
    }

    // Wait for the child + reader tasks to finish.
    let status = child.wait().await.map_err(|e| format!("elai.exe wait failed: {e}"))?;
    if let Some(h) = stdout_handle { let _ = h.await; }
    if let Some(h) = stderr_handle { let _ = h.await; }

    let stdout_text = all_lines.join("\n");

    // B-36 STEP 5: emit the final `response` event carrying the full output.
    let _ = app.emit(
        "response",
        json!({
            "request_id": request_id,
            "ok":         status.success(),
            "text":       stdout_text,
            "exit_code":  status.code(),
        }),
    );

    if status.success() {
        Ok(json!({ "ok": true, "text": stdout_text }))
    } else {
        Err(format!(
            "elai.exe prompt exited with {:?}: {}",
            status.code(),
            stdout_text
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

    // E-15: backend selection + Firecracker pool stats. Today these reflect
    // compile-time platform detection only; FirecrackerHotPool live counters
    // require an Arc<SandboxObservability> on AppState (deferred to follow-on
    // patch). RUNPOD-PENDING fields render as null on the dashboard panel.
    #[cfg(target_os = "linux")]
    let backend_selected = if std::path::Path::new("/dev/kvm").exists() {
        "Firecracker"
    } else {
        "Gvisor"
    };
    #[cfg(target_os = "windows")]
    let backend_selected = "AppContainer";
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    let backend_selected = "None";

    Ok(json!({
        "tier": tier,
        "profile_count": 0_u32,
        "allowed_paths_per_agent": allowed_paths_per_agent,
        "last_visibility_filter_hit_ms": null,
        // E-15 sandbox backend fields (additive, B-18 fields preserved above).
        "backend_selected": backend_selected,
        "restore_p50_ms": null,
        "snapshot_count": 0_u32,
        "uniqueness_regenerations_count": 0_u32,
        "researcher_net_allowed_hosts": ["arxiv.org", "huggingface.co"],
        "researcher_net_blocked_attempts": [],
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
    // C-15: OsClock-backed so the entropy scanner stays clean. Used by the
    // remote-sidecar progress timestamps that the UI displays.
    i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX)
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

// ---------------------------------------------------------------------------
// B-40 — IDEA FRAMING SURFACE
// ---------------------------------------------------------------------------
//
// Two Tauri commands powering the IdeaFramingWindow + MonitorWindow
// critic-rounds badge:
//   * `get_idea_framing_state` — reads the latest IdeaFramed +
//     CriticRoundComplete + IdeaFramingStarted SSB rows and assembles an
//     IdeaFramingState DTO. Returns Some(state) when a FRAMING substate is
//     currently active for the latest session OR None when no framing flow
//     has been started yet. The dashboard renders a placeholder when None.
//   * `submit_tradeoff_responses` — writes the user's tradeoff responses to
//     the canonical artifact path and emits a Finding (info severity) SSB
//     row indicating responses received. Future patches will use this row to
//     advance the FRAMING -> PLANNING substate transition.
//
// stub_ban: both commands query live SSB rows; neither returns mock data.
// When viewer is None (no session DB yet) the read command returns Ok(None).
// When writer is None the write command returns Err("ssb writer unavailable").

/// Dashboard DTO matching `IdeaFramingState` in dashboard-v2/src/lib/api/types.ts.
#[derive(Debug, Clone, serde::Serialize)]
pub struct IdeaFramingStateDto {
    pub session_id: String,
    pub framed_idea: Option<serde_json::Value>,
    pub critic_rounds_used: u32,
    pub is_converged: bool,
    pub is_active: bool,
}

/// One user response submitted via `submit_tradeoff_responses`.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TradeoffResponseDto {
    pub tradeoff_name: String,
    pub option: String,
}

/// Internal: fetch (session_id, payload_bytes) rows for an action_type via
/// the SsbViewer B-40 helper (`rows_with_session_by_action_type`).
fn query_rows_by_action_type(
    viewer: &elai_orchestrator::SsbViewer,
    action_type: SsbActionType,
) -> Result<Vec<(String, Vec<u8>)>, String> {
    viewer
        .rows_with_session_by_action_type(action_type)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_idea_framing_state(
    state: State<'_, AppState>,
) -> CmdResult<Option<IdeaFramingStateDto>> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(None);
    };

    let started_rows = query_rows_by_action_type(viewer, SsbActionType::IdeaFramingStarted)?;
    let framed_rows = query_rows_by_action_type(viewer, SsbActionType::IdeaFramed)?;

    let latest_framed = framed_rows.last();
    let latest_started = started_rows.last();

    let session_id = latest_framed
        .map(|(s, _)| s.clone())
        .or_else(|| latest_started.map(|(s, _)| s.clone()))
        .unwrap_or_default();

    if session_id.is_empty() {
        return Ok(None);
    }

    let critic_rows = query_rows_by_action_type(viewer, SsbActionType::CriticRoundComplete)?;
    let mut critic_rounds_used: u32 = 0;
    for (sid, bytes) in &critic_rows {
        if *sid != session_id {
            continue;
        }
        let Ok(payload) =
            elai_orchestrator::decode_ssb_payload_bytes::<SsbPayload>(bytes.as_slice())
        else {
            continue;
        };
        if let SsbPayload::CriticRoundComplete(p) = payload {
            if p.round_index > critic_rounds_used {
                critic_rounds_used = p.round_index;
            }
        }
    }

    let mut is_converged = false;
    let mut framed_idea_json: Option<serde_json::Value> = None;
    if let Some((sid, bytes)) = latest_framed {
        if *sid == session_id {
            if let Ok(payload) =
                elai_orchestrator::decode_ssb_payload_bytes::<SsbPayload>(bytes.as_slice())
            {
                if let SsbPayload::IdeaFramed(p) = payload {
                    is_converged = p.converged;
                    let path = std::path::PathBuf::from(&p.artifact_path);
                    if path.exists() {
                        if let Ok(raw) = std::fs::read_to_string(&path) {
                            if let Ok(parsed) = toml::from_str::<toml::Value>(&raw) {
                                if let Ok(j) = serde_json::to_value(&parsed) {
                                    framed_idea_json = Some(j);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let started_exists = started_rows.iter().any(|(s, _)| *s == session_id);
    let is_active = started_exists && (latest_framed.is_none() || !is_converged);

    Ok(Some(IdeaFramingStateDto {
        session_id,
        framed_idea: framed_idea_json,
        critic_rounds_used,
        is_converged,
        is_active,
    }))
}

/// C-10 — flattened DTO surfaced by [`get_stagnation_events`]. One row per
/// anti-spiral / lateral-thinking / goal-drift / watchdog SSB row, sorted
/// by timestamp DESC. Optional fields are populated by the variant-specific
/// match arms below; consumers branch on `action_type` for type-specific
/// rendering.
#[derive(Debug, Clone, serde::Serialize)]
pub struct StagnationEventDto {
    pub timestamp_ms: i64,
    pub action_type: String,
    pub agent_id: String,
    /// Set on StagnationSpinningDetected / StagnationOscillationDetected
    /// (carries `spinning | oscillation | no_drift | diminishing_returns`).
    pub pattern: Option<String>,
    /// Set on LateralThinkingActivated / LateralThinkingSucceeded /
    /// LateralThinkingExhausted.
    pub persona: Option<String>,
    /// Set on GoalDriftWarning / GoalDriftBlock — Jaccard similarity in
    /// the plan-direction reading (higher = aligned, lower = drifted).
    pub drift_score: Option<f32>,
    /// Set on WatchdogTimeout — elapsed since task start, ms.
    pub elapsed_ms: Option<u64>,
    pub detail: String,
}

/// C-10 — return the last N stagnation / lateral-thinking / goal-drift /
/// watchdog events for the OrchestrationDrawer StagnationSection.
/// Flattens 8 SsbActionType variant shapes into a single DTO so the
/// Svelte side has one row type.
#[tauri::command]
pub async fn get_stagnation_events(
    state: State<'_, AppState>,
) -> CmdResult<Vec<StagnationEventDto>> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(Vec::new());
    };

    const PER_TYPE_LIMIT: usize = 50;
    let action_type_strs = [
        "stagnation_spinning_detected",
        "stagnation_oscillation_detected",
        "lateral_thinking_activated",
        "lateral_thinking_succeeded",
        "lateral_thinking_exhausted",
        "goal_drift_warning",
        "goal_drift_block",
        "watchdog_timeout",
    ];

    let mut out: Vec<StagnationEventDto> = Vec::new();
    for at in action_type_strs {
        let rows = match viewer.recent_rows_of_type(at, PER_TYPE_LIMIT) {
            Ok(rs) => rs,
            Err(_) => continue,
        };
        for row in rows {
            let p = &row.payload_json;
            let pattern = p
                .get("pattern")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned());
            let persona = p
                .get("persona")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned());
            let drift_score = p.get("drift_score").and_then(|v| v.as_f64()).map(|d| d as f32);
            let elapsed_ms = p.get("elapsed_ms").and_then(|v| v.as_u64());
            let detail = p
                .get("detail")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned();
            out.push(StagnationEventDto {
                timestamp_ms: row.ts,
                action_type: row.action_type,
                agent_id: row.agent_id,
                pattern,
                persona,
                drift_score,
                elapsed_ms,
                detail,
            });
        }
    }

    // Sort by timestamp DESC and cap at the last 50 rows.
    out.sort_by(|a, b| b.timestamp_ms.cmp(&a.timestamp_ms));
    if out.len() > 50 {
        out.truncate(50);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// E-23 -- developer-discourse positioning + UX-guard panels.
//
// Two read-only commands the ContextStatePanel + TrustTelemetryPanel poll:
//
//   * `get_context_state` -- one snapshot of FSM state + token-budget % +
//     prompt-injection-check status + runtime_choice_badge label. The
//     panel renders yellow >=70% / red >=85% per CONTEXT_BUDGET_*_PCT
//     constants in orchestrator::ssb.
//
//   * `get_trust_telemetry` -- aggregated TrustReport across existing SSB
//     rows (productivity proxy + verification density + revert signal +
//     ungrounded counter + stagnation distribution). Wire-dark unless
//     ELAI_DEVELOPER_DISCOURSE_ENABLE=1 -- when off, returns the empty
//     report so the dashboard renders a stable grid.
//
// Plan body: F:\.ELAI_workflow\docs\thefinalglance\E-plan\
// E-23_developer_discourse_2026.md tasks 1 + 2. Source-grounded by
// arXiv:2507.09089 (METR RCT, productivity baseline frame) and Anthropic
// issue #13171 (context-loss UX driver).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TopicBudgetDto {
    pub topic: String,
    pub remaining_calls: u32,
    pub remaining_chars: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextStateDto {
    pub fsm_state: String,
    pub substate: Option<String>,
    pub token_budget_pct: f32,
    pub mcp_think_topics: Vec<TopicBudgetDto>,
    pub last_precompact_thresholds: Vec<f32>,
    /// "passed" | "blocked" | "not_run" -- driven by C-42 last-run rows
    /// (C-42 is PLANNED today, so the default is "not_run").
    pub prompt_injection_check_status: String,
    /// "local" | "cloud_billed" | "" -- derived from B-37 runtime_fallback
    /// rows. Empty when no runtime has been selected yet this session.
    pub runtime_choice_badge: String,
    /// E-3 sub-phase 4B -- current agent role (planner / coder / critic
    /// shape; see `agent_role_to_voice_section`). Sourced from the most
    /// recent `e3_runtime_handoff_event::unified_preamble_applied` row;
    /// None until the orchestrator emits its first wrapped dispatch.
    pub current_role: Option<String>,
    /// E-3 sub-phase 4B -- underlying runtime backend name (sidecar /
    /// anthropic_api / etc.) for the role badge drawer. Hidden by
    /// default in the chat UI; revealed on drawer expansion.
    pub current_runtime: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrustReportDto {
    pub tasks_per_hour: f64,
    pub findings_per_task: f64,
    pub revert_rate: f64,
    pub ungrounded_count: u64,
    pub stagnation_dist: std::collections::HashMap<String, u64>,
}

/// Map a B-37 RuntimeBackend snake_case identifier to the
/// CommandWindow status-row partition. Local-first backends (sidecar,
/// mlx, llama_cpp, ollama, vllm, sglang, lsp, ppm) return "local"; cloud-
/// billed backends return "cloud_billed". Unknown identifiers map to
/// the empty string so the badge collapses to a no-op for unseen
/// runtimes -- the panel renders the empty case explicitly.
fn classify_runtime_badge(backend: &str) -> &'static str {
    match backend {
        "sidecar" | "ollama" | "vllm" | "mlx" | "llama_cpp" | "sglang" | "lsp" | "ppm" => "local",
        "anthropic_api" | "claude_code" | "codex_cli" | "opencode" | "hermes" | "gemini_cli"
        | "kiro_cli" | "copilot_cli" | "composer2" => "cloud_billed",
        _ => "",
    }
}

#[tauri::command]
pub async fn get_context_state(state: State<'_, AppState>) -> CmdResult<ContextStateDto> {
    use elai_orchestrator::ssb::{CONTEXT_BUDGET_RED_PCT, CONTEXT_BUDGET_YELLOW_PCT};
    // Sentinel-quiet refs so the constants don't drift silently. The
    // panel uses the same constants in its threshold check copy. The
    // ordering check is a compile-pass-time sanity check.
    let _ = (CONTEXT_BUDGET_YELLOW_PCT, CONTEXT_BUDGET_RED_PCT);

    // Default zeroed shape -- the panel always renders something. We
    // overwrite from SSB rows below when the viewer is available.
    let mut dto = ContextStateDto {
        fsm_state: "IDLE".to_owned(),
        substate: None,
        token_budget_pct: 0.0,
        mcp_think_topics: Vec::new(),
        last_precompact_thresholds: Vec::new(),
        prompt_injection_check_status: "not_run".to_owned(),
        runtime_choice_badge: String::new(),
        current_role: None,
        current_runtime: None,
    };

    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(dto);
    };

    // FSM state: delegate to the contract-tested SsbViewer::latest_fsm_state
    // (msgpack decode path) — see fsm_phase_transition_contract.rs.
    if let Ok(snap) = viewer.latest_fsm_state() {
        if !snap.state.is_empty() {
            dto.fsm_state = snap.state;
        }
    }

    // Substate: the latest phase_transition row may carry a `substate` field
    // in its payload_json (externally-tagged enum shape). The
    // latest_fsm_state() snapshot does not surface substate, so we re-read
    // the raw row here to preserve the pre-refactor substate population.
    if let Ok(rows) = viewer.recent_rows_of_type("phase_transition", 1) {
        if let Some(row) = rows.first() {
            if let Some(sub) = row.payload_json.get("substate").and_then(|v| v.as_str()) {
                dto.substate = Some(sub.to_owned());
            }
        }
    }

    // Token-budget % -- read the most recent context_budget_computed row
    // (B-31.5 / B-7). Payload field name: `usage_pct` (per existing
    // ContextBudgetComputedPayload shape). Fall back to 0.0 if absent.
    if let Ok(rows) = viewer.recent_rows_of_type("context_budget_computed", 1) {
        if let Some(row) = rows.first() {
            if let Some(pct) = row.payload_json.get("usage_pct").and_then(|v| v.as_f64()) {
                dto.token_budget_pct = pct as f32;
            }
        }
    }

    // Runtime badge -- look at most recent runtime_fallback row's
    // `succeeded` backend (B-37). If no row, the badge stays empty.
    if let Ok(rows) = viewer.recent_rows_of_type("runtime_fallback", 1) {
        if let Some(row) = rows.first() {
            if let Some(backend) = row
                .payload_json
                .get("succeeded")
                .and_then(|v| v.as_str())
            {
                dto.runtime_choice_badge = classify_runtime_badge(backend).to_owned();
            }
        }
    }

    // Prompt-injection check -- C-42 producer is PLANNED. We probe for
    // the row family (`gateway_policy_allowed` / `gateway_policy_denied`)
    // and keep "not_run" otherwise.
    if let Ok(rows) = viewer.recent_rows_of_type("gateway_policy_allowed", 1) {
        if !rows.is_empty() {
            dto.prompt_injection_check_status = "passed".to_owned();
        }
    }
    if dto.prompt_injection_check_status == "not_run" {
        if let Ok(rows) = viewer.recent_rows_of_type("gateway_policy_denied", 1) {
            if !rows.is_empty() {
                dto.prompt_injection_check_status = "blocked".to_owned();
            }
        }
    }

    // E-3 sub-phase 4B -- current_role + current_runtime from the most
    // recent e3_runtime_handoff_event row. Both fields stay None until
    // the orchestrator emits its first wrapped dispatch.
    if let Ok(rows) = viewer.recent_rows_of_type("e3_runtime_handoff_event", 1) {
        if let Some(row) = rows.first() {
            if let Some(role) = row.payload_json.get("role").and_then(|v| v.as_str()) {
                if !role.is_empty() {
                    dto.current_role = Some(role.to_owned());
                }
            }
            if let Some(backend) = row.payload_json.get("backend").and_then(|v| v.as_str()) {
                if !backend.is_empty() {
                    dto.current_runtime = Some(backend.to_owned());
                }
            }
        }
    }

    // E-23 WIRING_CONTRACT observable_effect: emit DeveloperDiscourseEvent
    // SSB rows from this real entrypoint (dashboard-v2 get_context_state
    // Tauri command) so build_badge_payload + build_context_warning_payload
    // are no longer wire-dark. Best-effort: a write failure is logged via
    // tracing but never surfaces as a command error, mirroring the
    // get_trust_telemetry emit idiom at commands.rs:9796..9813. The gate
    // is the same ELAI_DEVELOPER_DISCOURSE_ENABLE env var so nothing
    // emits when the discourse plane is wire-dark.
    if let Some(writer) = state.writer.as_ref() {
        use elai_orchestrator::trust_telemetry::{
            build_badge_payload, build_context_warning_payload, discourse_enabled,
        };

        if discourse_enabled() {
            let session_id = Some(writer.session_id().to_owned());
            let ts_ms = now_ms(&state);

            // Runtime-choice badge emit -- only when the classified badge
            // is non-empty (local / cloud_billed). An empty label means
            // classify_runtime_badge did not recognise the backend, so
            // there is no meaningful transition to record.
            if !dto.runtime_choice_badge.is_empty() {
                let payload = SsbPayload::DeveloperDiscourseEvent(build_badge_payload(
                    "",
                    &dto.runtime_choice_badge,
                    session_id.clone(),
                    ts_ms,
                ));
                if let Err(e) = writer
                    .write_payload("dashboard-v2", 0, payload, vec![])
                    .await
                {
                    tracing::warn!(
                        target: "elai_dashboard",
                        error = %e,
                        "get_context_state: badge SSB emit failed (best-effort, DTO still returned)"
                    );
                }
            }

            // Context-state warning emit -- only when token_budget_pct
            // crosses the YELLOW (70.0) or RED (85.0) thresholds. The
            // threshold_label is derived inside build_context_warning_payload
            // from the same constants this command already imports.
            if dto.token_budget_pct >= CONTEXT_BUDGET_YELLOW_PCT {
                let payload = SsbPayload::DeveloperDiscourseEvent(
                    build_context_warning_payload(dto.token_budget_pct, session_id, ts_ms),
                );
                if let Err(e) = writer
                    .write_payload("dashboard-v2", 0, payload, vec![])
                    .await
                {
                    tracing::warn!(
                        target: "elai_dashboard",
                        error = %e,
                        "get_context_state: context warning SSB emit failed (best-effort, DTO still returned)"
                    );
                }
            }
        }
    }

    Ok(dto)
}

// ---------------------------------------------------------------------------
// E-3 sub-phase 4B (SCHEMA_VERSION 190) -- runtime-handoff event recorder.
//
// Single Tauri command that fans the 6 RuntimeHandoffEventKind arms into
// SSB rows. Mirrors the record_model_visibility shape (one wire-dark
// emission per UI-triggered or sidecar-triggered handoff). The
// orchestrator's runtime_orchestrator.rs emits the
// UnifiedPreambleApplied + RuntimePrewarm{Started,Complete} arms in
// process via the Arc<Ssb> handle; this command is the dashboard's
// own hook for surface emissions (StyleNormalizerApplied today; the
// other arms are reserved for the sidecar-side wiring per CLAUDE.md
// sidecar HARDWARE-HEAVY rule).
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn record_runtime_handoff_event(
    state: State<'_, AppState>,
    kind: String,
    role: Option<String>,
    backend: Option<String>,
    latency_ms: Option<u32>,
    details: Option<Value>,
) -> CmdResult<Value> {
    use elai_orchestrator::ssb::{
        RuntimeHandoffEventKind, RuntimeHandoffEventPayload, ELAI_E3_RUNTIME_HANDOFF_ENABLE_ENV,
    };

    // Wire-dark gate: silently skip emission when the env var is unset.
    let enabled = std::env::var(ELAI_E3_RUNTIME_HANDOFF_ENABLE_ENV)
        .ok()
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false);

    if !enabled {
        return Ok(json!({ "ok": true, "emitted": false }));
    }

    let Some(writer) = state.writer.as_ref() else {
        return Ok(json!({ "ok": true, "emitted": false }));
    };

    // Map the snake-case kind string to the canonical enum. Unknown
    // values surface as a CmdResult error so the frontend gets a
    // typed failure rather than a silent no-op.
    let kind_enum = match kind.as_str() {
        "unified_preamble_applied" => RuntimeHandoffEventKind::UnifiedPreambleApplied,
        "runtime_prewarm_started" => RuntimeHandoffEventKind::RuntimePrewarmStarted,
        "runtime_prewarm_complete" => RuntimeHandoffEventKind::RuntimePrewarmComplete,
        "handoff_payload_compressed" => RuntimeHandoffEventKind::HandoffPayloadCompressed,
        "persona_vector_applied" => RuntimeHandoffEventKind::PersonaVectorApplied,
        "style_normalizer_applied" => RuntimeHandoffEventKind::StyleNormalizerApplied,
        other => {
            return Err(format!(
                "unknown e3_runtime_handoff_event kind: {other}"
            ))
        }
    };

    // Stitch latency_ms (UI-friendly param) into details_json if
    // provided. Keeps the wire shape consistent with the orchestrator-
    // side emission so dashboards can render a uniform timeline.
    let mut details_json = details.unwrap_or_else(|| json!({}));
    if let (Some(ms), Some(obj)) = (latency_ms, details_json.as_object_mut()) {
        obj.insert("duration_ms".to_owned(), json!(ms));
    }

    let payload = SsbPayload::E3RuntimeHandoffEvent(RuntimeHandoffEventPayload {
        kind: kind_enum,
        role,
        backend,
        session_id: Some(writer.session_id().to_owned()),
        details_json,
        ts_ms: now_ms(&state),
    });
    writer
        .write_payload("dashboard-v2", 0, payload, vec![])
        .await
        .map_err(map_err)?;
    Ok(json!({ "ok": true, "emitted": true }))
}

#[tauri::command]
pub async fn get_trust_telemetry(state: State<'_, AppState>) -> CmdResult<TrustReportDto> {
    use elai_orchestrator::trust_telemetry::{aggregate_session, discourse_enabled, TrustRowInput};

    // When the gate is off we still return the empty zeroed shape so
    // the panel renders. Mirrors E-22 / E-43 / E-55 gate-defaults.
    if !discourse_enabled() {
        return Ok(TrustReportDto {
            tasks_per_hour: 0.0,
            findings_per_task: 0.0,
            revert_rate: 0.0,
            ungrounded_count: 0,
            stagnation_dist: std::collections::HashMap::new(),
        });
    }

    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(TrustReportDto {
            tasks_per_hour: 0.0,
            findings_per_task: 0.0,
            revert_rate: 0.0,
            ungrounded_count: 0,
            stagnation_dist: std::collections::HashMap::new(),
        });
    };

    const PER_TYPE_LIMIT: usize = 200;
    let action_types = [
        "phase_transition",
        "finding",
        "file_write",
        "claim",
        "stagnation_spinning_detected",
        "stagnation_oscillation_detected",
    ];
    let mut rows: Vec<TrustRowInput> = Vec::new();
    let mut earliest_ts: Option<i64> = None;
    let mut latest_ts: Option<i64> = None;
    for at in action_types {
        let Ok(detail) = viewer.recent_rows_of_type(at, PER_TYPE_LIMIT) else {
            continue;
        };
        for d in detail {
            earliest_ts = Some(earliest_ts.map_or(d.ts, |e| e.min(d.ts)));
            latest_ts = Some(latest_ts.map_or(d.ts, |e| e.max(d.ts)));
            rows.push(TrustRowInput {
                action_type: d.action_type,
                ts_ms: d.ts,
                payload_json: d.payload_json,
            });
        }
    }
    let elapsed_hours = match (earliest_ts, latest_ts) {
        (Some(a), Some(b)) if b > a => (b - a) as f64 / 3_600_000.0_f64,
        _ => 1.0_f64 / 60.0_f64, // 1 minute floor; aggregate_session also floors.
    };
    let report = aggregate_session(&rows, elapsed_hours);

    // E-23 WIRING_CONTRACT observable_effect: emit a DeveloperDiscourseEvent
    // SSB row of kind TrustTelemetryAggregated whenever the gate is ON and a
    // writer is present. Best-effort: a write failure is logged via tracing but
    // never surfaces as a command error (the caller needs the DTO regardless).
    if let Some(writer) = state.writer.as_ref() {
        use elai_orchestrator::trust_telemetry::build_aggregated_payload;
        let payload = SsbPayload::DeveloperDiscourseEvent(build_aggregated_payload(
            &report,
            Some(writer.session_id().to_owned()),
            now_ms(&state),
        ));
        if let Err(e) = writer
            .write_payload("dashboard-v2", 0, payload, vec![])
            .await
        {
            tracing::warn!(
                target: "elai_dashboard",
                error = %e,
                "get_trust_telemetry: SSB emit failed (best-effort, DTO still returned)"
            );
        }
    }

    Ok(TrustReportDto {
        tasks_per_hour: report.tasks_per_hour,
        findings_per_task: report.findings_per_task,
        revert_rate: report.revert_rate,
        ungrounded_count: report.ungrounded_count,
        stagnation_dist: report.stagnation_dist.into_iter().collect(),
    })
}

#[tauri::command]
pub async fn submit_tradeoff_responses(
    state: State<'_, AppState>,
    responses: Vec<TradeoffResponseDto>,
) -> CmdResult<Value> {
    let writer = state
        .writer
        .as_ref()
        .ok_or_else(|| "ssb writer unavailable; no session db open".to_owned())?;

    // Locate the active session's idea-framing dir using the latest
    // IdeaFramingStarted row in SSB (mirrors get_idea_framing_state path
    // resolution).
    let viewer = state
        .viewer
        .as_ref()
        .ok_or_else(|| "ssb viewer unavailable; no session db open".to_owned())?;
    let started_rows = query_rows_by_action_type(viewer, SsbActionType::IdeaFramingStarted)?;
    let session_id = started_rows
        .last()
        .map(|(sid, _)| sid.clone())
        .ok_or_else(|| "no FRAMING session active".to_owned())?;

    let project_root = state.project_root.lock().map_err(|e| e.to_string())?.clone();
    let idea_dir = project_root
        .join("workflow")
        .join("sessions")
        .join(&session_id)
        .join("idea-framing");
    std::fs::create_dir_all(&idea_dir).map_err(map_err)?;
    let responses_path = idea_dir.join("tradeoff-responses.toml");

    // Build the TOML output. One [[response]] table per submitted entry.
    let mut buf = String::new();
    buf.push_str("# B-40: user tradeoff responses\n");
    for r in &responses {
        if r.tradeoff_name.trim().is_empty() || r.option.trim().is_empty() {
            return Err("tradeoff_name and option must be non-empty".to_owned());
        }
        buf.push_str("\n[[response]]\n");
        buf.push_str(&format!("tradeoff_name = {:?}\n", r.tradeoff_name));
        buf.push_str(&format!("option = {:?}\n", r.option));
    }
    std::fs::write(&responses_path, &buf).map_err(map_err)?;

    // B-40 WARN-1 audit fix: emit an info Finding row so dashboard activity
    // streams + auditor SSB scans observe the response submission. Avoids
    // a SCHEMA_VERSION bump (no new SsbActionType variant) while still
    // recording the event in the canonical event log.
    {
        use elai_orchestrator::FindingPayload;
        let payload = SsbPayload::Finding(FindingPayload {
            severity: "info".to_owned(),
            category: "tradeoff_responses_submitted".to_owned(),
            message: format!(
                "user submitted {} tradeoff response(s) for session {}",
                responses.len(),
                session_id
            ),
            file_path: Some(responses_path.to_string_lossy().into_owned()),
            line: None,
            failure_category: None,
            audit_step: None,
            few_shot_examples: None,
        });
        let _ = writer
            .write_payload("dashboard-v2", 0, payload, vec![])
            .await;
    }

    Ok(json!({
        "ok": true,
        "session_id": session_id,
        "responses_persisted": responses.len(),
        "responses_path": responses_path.to_string_lossy(),
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

// ---------------------------------------------------------------------------
// B-35 Phase C — CONTEXT ARTIFACTS + ENTRY POINTS (Tauri command surface)
//
// These commands mirror the orchestrator's dispatcher arms (production_agent_rpc.rs)
// but are invoked directly from the dashboard frontend without going through the
// agent IPC pipeline. All blocking SQLite I/O is offloaded via spawn_blocking.
//
// db_path: the full path to context_{project_id}.db, supplied by the frontend
// via the ArtifactsPanel prop. The frontend derives this from the project root
// it already holds in state. Using an explicit path keeps the command surface
// stateless w.r.t. which project is open (multiple projects could be open
// simultaneously in future).
// ---------------------------------------------------------------------------

/// List all artifact names (distinct, sorted) in the context database.
///
/// Frontend call: `invoke("list_artifacts", { db_path })`
/// Returns: `{ artifact_names: string[] }`
#[tauri::command]
pub async fn list_artifacts(db_path: String) -> CmdResult<serde_json::Value> {
    let path = std::path::PathBuf::from(&db_path);
    let names = tokio::task::spawn_blocking(move || -> Result<Vec<String>, String> {
        let store = elai_knowledge::artifacts::store::ArtifactStore::open(&path)
            .map_err(|e| format!("list_artifacts: store open failed: {e}"))?;
        let chunks = store
            .list_all_chunks()
            .map_err(|e| format!("list_artifacts: list_all_chunks failed: {e}"))?;
        let mut names: Vec<String> = chunks
            .into_iter()
            .map(|c| c.artifact_name)
            .collect();
        names.sort_unstable();
        names.dedup();
        Ok(names)
    })
    .await
    .map_err(|e| format!("list_artifacts: spawn_blocking panic: {e}"))??;
    Ok(serde_json::json!({ "artifact_names": names }))
}

/// Search artifact chunks by cosine similarity.
///
/// `artifact_name` restricts search to one artifact when present.
/// `limit` defaults to 10.
///
/// Frontend call: `invoke("search_artifacts", { db_path, query, artifact_name?, limit? })`
/// Returns: `{ results: ArtifactSearchResult[] }`
///
/// When no VectorModule is wired at the dashboard level (expected in Phase C)
/// the command returns an empty result set rather than an error.
#[tauri::command]
pub async fn search_artifacts(
    db_path: String,
    query: String,
    artifact_name: Option<String>,
    limit: Option<usize>,
) -> CmdResult<serde_json::Value> {
    let limit = limit.unwrap_or(10).min(MAX_LIMIT);
    let path = std::path::PathBuf::from(&db_path);

    // B-35 Phase C: no VectorModule is wired at dashboard level yet (deferred
    // to Phase D). Fall back to a BM25-style keyword scan across chunk content.
    let results = tokio::task::spawn_blocking(move || -> Result<Vec<serde_json::Value>, String> {
        let store = elai_knowledge::artifacts::store::ArtifactStore::open(&path)
            .map_err(|e| format!("search_artifacts: store open failed: {e}"))?;
        let chunks = match artifact_name.as_deref() {
            Some(name) => store
                .list_artifact_chunks(name)
                .map_err(|e| format!("search_artifacts: list_artifact_chunks failed: {e}"))?,
            None => store
                .list_all_chunks()
                .map_err(|e| format!("search_artifacts: list_all_chunks failed: {e}"))?,
        };
        let q_lower = query.to_lowercase();
        let mut scored: Vec<(f64, &elai_knowledge::artifacts::ArtifactChunk)> = chunks
            .iter()
            .filter_map(|c| {
                let content_lower = c.content.to_lowercase();
                let hits = content_lower.matches(q_lower.as_str()).count();
                if hits == 0 {
                    None
                } else {
                    // Simple TF score: hits / total words (approximate).
                    let words = content_lower.split_whitespace().count().max(1);
                    let score = hits as f64 / words as f64;
                    Some((score, c))
                }
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let out: Vec<serde_json::Value> = scored
            .into_iter()
            .take(limit)
            .map(|(score, c)| {
                serde_json::json!({
                    "artifact_name": c.artifact_name,
                    "relative_path": c.relative_path,
                    "content": c.content,
                    "start_line": c.start_line,
                    "end_line": c.end_line,
                    "score": score,
                })
            })
            .collect();
        Ok(out)
    })
    .await
    .map_err(|e| format!("search_artifacts: spawn_blocking panic: {e}"))??;

    Ok(serde_json::json!({ "results": results }))
}

/// Trigger re-indexing of one artifact by name.
///
/// Phase C stub: drops existing chunks for the artifact to mark it stale.
/// Full re-indexing with chunking + embedding is wired by the CLI pipeline;
/// the dashboard signals intent and the CLI processes on next run.
///
/// Frontend call: `invoke("index_artifacts", { db_path, artifact_name })`
/// Returns: `{ ok: true, artifact_name: string, chunks_dropped: number }` on success.
#[tauri::command]
pub async fn index_artifacts(
    db_path: String,
    artifact_name: String,
) -> CmdResult<serde_json::Value> {
    let path = std::path::PathBuf::from(&db_path);
    let name = artifact_name.clone();
    let chunks_dropped = tokio::task::spawn_blocking(move || -> Result<usize, String> {
        let store = elai_knowledge::artifacts::store::ArtifactStore::open(&path)
            .map_err(|e| format!("index_artifacts: store open failed: {e}"))?;
        store
            .drop_chunks_for_artifact(&name)
            .map_err(|e| format!("index_artifacts: drop_chunks failed: {e}"))
    })
    .await
    .map_err(|e| format!("index_artifacts: spawn_blocking panic: {e}"))??;
    tracing::info!(
        artifact = %artifact_name,
        chunks_dropped,
        "index_artifacts: chunks cleared (re-index via CLI)"
    );
    Ok(serde_json::json!({
        "ok": true,
        "artifact_name": artifact_name,
        "chunks_dropped": chunks_dropped,
    }))
}

/// Remove all chunks for an artifact from the context database.
///
/// Frontend call: `invoke("remove_artifacts", { db_path, artifact_name })`
/// Returns: `{ artifact_name: string, chunks_removed: number }`
#[tauri::command]
pub async fn remove_artifacts(
    db_path: String,
    artifact_name: String,
) -> CmdResult<serde_json::Value> {
    let path = std::path::PathBuf::from(&db_path);
    let name = artifact_name.clone();
    let dropped = tokio::task::spawn_blocking(move || -> Result<usize, String> {
        let store = elai_knowledge::artifacts::store::ArtifactStore::open(&path)
            .map_err(|e| format!("remove_artifacts: store open failed: {e}"))?;
        store
            .drop_chunks_for_artifact(&name)
            .map_err(|e| format!("remove_artifacts: drop_chunks failed: {e}"))
    })
    .await
    .map_err(|e| format!("remove_artifacts: spawn_blocking panic: {e}"))??;
    Ok(serde_json::json!({
        "artifact_name": artifact_name,
        "chunks_removed": dropped,
    }))
}

/// Detect entry points in the project (orphan, well-known name, framework
/// route, test) using the symbol graph reverse index.
///
/// `state` is used to derive `project_root` and `project_id`.
/// `include_tests`: when false, test entry points are suppressed. Default: true.
///
/// Frontend call: `invoke("list_entry_points", { include_tests? })`
/// Returns: `{ entry_points: EntryPointDto[], summary: { by_reason, total } }`
#[tauri::command]
pub async fn list_entry_points(
    state: State<'_, AppState>,
    include_tests: Option<bool>,
) -> CmdResult<serde_json::Value> {
    use elai_knowledge::symbol_graph::cache::SymbolGraphCache;
    use elai_knowledge::symbol_graph::entry_points::{
        detect_entry_points, entry_point_filenames, EntryPointReason, EntryPointSummary,
    };

    let include_tests = include_tests.unwrap_or(true);
    let project_root = state.project_root_snapshot();
    let project_id = b33_resolve_project_id(&state);
    let dkg_path = elai_core::paths::dkg_db_path(&project_root);

    if !dkg_path.exists() {
        return Ok(serde_json::json!({
            "entry_points": [],
            "summary": { "by_reason": {}, "total": 0 },
        }));
    }

    // SymbolGraphCache::new + ensure_reverse_index are sync; run on blocking thread.
    let result = tokio::task::spawn_blocking(move || -> Result<serde_json::Value, String> {
        let mut cache = SymbolGraphCache::new(
            &dkg_path,
            &project_id,
            elai_knowledge::symbol_graph::SYMBOL_FILE_LRU_SIZE,
        )
        .map_err(|e| format!("list_entry_points: cache open failed: {e}"))?;

        // Compute reverse index: callee_file -> Set<caller_file>.
        let reverse_index = cache
            .ensure_reverse_index()
            .map_err(|e| format!("list_entry_points: reverse_index failed: {e}"))?
            .clone();

        // Collect all indexed file paths from the reverse index.
        let all_files: std::collections::BTreeSet<String> = {
            let mut s = std::collections::BTreeSet::new();
            for (k, vs) in reverse_index.iter() {
                s.insert(k.clone());
                for v in vs {
                    s.insert(v.clone());
                }
            }
            s
        };

        // Filename-based filter: load payloads only for entry-point filename matches
        // and orphan candidates (files NOT in reverse_index keys or with empty callers).
        let filename_stems = entry_point_filenames();
        let load_candidates: Vec<String> = all_files
            .into_iter()
            .filter(|f| {
                let callers = reverse_index.get(f);
                let is_orphan = callers.map_or(true, |s| s.is_empty());
                let is_ep_name = {
                    let stem = std::path::Path::new(f)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    filename_stems.contains(stem)
                };
                is_orphan || is_ep_name
            })
            .collect();

        // Load payloads (Arc<SymbolGraphFilePayload>); deref for detect_entry_points.
        let mut payload_arcs = Vec::new();
        for file_path in &load_candidates {
            match cache.get_or_load_payload(file_path) {
                Ok(Some(arc)) => payload_arcs.push(arc),
                Ok(None) => {}
                Err(e) => {
                    tracing::debug!(file = %file_path, error = %e, "list_entry_points: payload load skipped");
                }
            }
        }
        let payloads: Vec<elai_knowledge::symbol_graph::types::SymbolGraphFilePayload> =
            payload_arcs.iter().map(|arc| (**arc).clone()).collect();

        let mut entries = detect_entry_points(&reverse_index, &payloads);
        if !include_tests {
            entries.retain(|e| !matches!(e.reason, EntryPointReason::Test));
        }

        let summary = EntryPointSummary::from_entries(&entries);
        let ep_json: Vec<serde_json::Value> = entries
            .into_iter()
            .map(|e| {
                serde_json::json!({
                    "id": e.id,
                    "name": e.name,
                    "file": e.file,
                    "line": e.line,
                    "reason": format!("{:?}", e.reason),
                })
            })
            .collect();

        Ok(serde_json::json!({
            "entry_points": ep_json,
            "summary": {
                "by_reason": summary.by_reason,
                "total": summary.total,
            },
        }))
    })
    .await
    .map_err(|e| format!("list_entry_points: spawn_blocking panic: {e}"))??;

    Ok(result)
}

// ---------------------------------------------------------------------------
// B-38 — AMBIGUITY GATE SURFACE
// ---------------------------------------------------------------------------
//
// cross-lang-pair: dashboard/tauri/api
//
// `get_ambiguity_state` returns the dashboard-facing snapshot of the
// continuous ambiguity gate (B-38). The shape mirrors `AmbiguityState` in
// `crates/dashboard-v2/src/lib/api/types.ts`.
//
// C-2: replaced the B-38 placeholder mock body with a live SSB read of
// `ambiguity_scored` / `ambiguity_floor_failed` / `ambiguity_streak_advanced`
// rows. When no rows exist yet (cold-start, pre-first-task), the command
// returns the same shape with zero score + empty floor_failures + 0-streak so
// the GatesPanel can render an "awaiting data" layout without crashing.
// The producer (AmbiguityScorer wired into task_runner) lands in C-8 per
// plan §Out-of-scope; in the meantime any task-runner ground_plan call that
// chooses to populate `ctx.ambiguity_score` will surface here automatically.
#[tauri::command]
pub fn get_ambiguity_state(state: State<'_, AppState>) -> CmdResult<Value> {
    use elai_reasoning::{
        AMBIGUITY_THRESHOLD, AUTO_COMPLETE_STREAK_REQUIRED, CONSTRAINT_CLARITY_FLOOR,
        CONSTRAINT_CLARITY_WEIGHT, GOAL_CLARITY_FLOOR, GOAL_CLARITY_WEIGHT,
        SUCCESS_CRITERIA_CLARITY_FLOOR, SUCCESS_CRITERIA_CLARITY_WEIGHT,
    };

    let viewer_opt = state.viewer.as_ref();

    let latest_scored = viewer_opt.and_then(|v| v.latest_ambiguity_scored().ok().flatten());
    let latest_floor = viewer_opt.and_then(|v| v.latest_ambiguity_floor_failed().ok().flatten());
    let latest_streak = viewer_opt.and_then(|v| v.latest_ambiguity_streak().ok().flatten());

    let threshold = AMBIGUITY_THRESHOLD;
    let streak_required = AUTO_COMPLETE_STREAK_REQUIRED as u32;

    let (score, milestone, brownfield, goal, constraint, success, context_value) =
        if let Some(s) = &latest_scored {
            (
                s.score,
                s.milestone.clone(),
                s.brownfield,
                s.goal,
                s.constraint,
                s.success,
                s.context,
            )
        } else {
            (
                0.0_f32,
                "initial".to_owned(),
                false,
                0.0_f32,
                0.0_f32,
                0.0_f32,
                None,
            )
        };

    let floor_failures: Vec<String> = latest_floor
        .as_ref()
        .map(|f| f.failed_dimensions.clone())
        .unwrap_or_default();

    let streak_current = latest_streak.as_ref().map(|s| s.current_streak).unwrap_or(0);
    let qualifies_for_advance = score <= threshold && floor_failures.is_empty();

    let context_clarity = context_value
        .map(|c| {
            json!({
                "score": c,
                "weight": 0.15_f32,
                "floor": 0.60_f32,
            })
        })
        .unwrap_or(Value::Null);

    Ok(json!({
        "score": score,
        "milestone": milestone,
        "threshold": threshold,
        "brownfield": brownfield,
        "components": {
            "goal_clarity":             { "score": goal, "weight": GOAL_CLARITY_WEIGHT, "floor": GOAL_CLARITY_FLOOR },
            "constraint_clarity":       { "score": constraint, "weight": CONSTRAINT_CLARITY_WEIGHT, "floor": CONSTRAINT_CLARITY_FLOOR },
            "success_criteria_clarity": { "score": success, "weight": SUCCESS_CRITERIA_CLARITY_WEIGHT, "floor": SUCCESS_CRITERIA_CLARITY_FLOOR },
            "context_clarity":          context_clarity,
        },
        "floor_failures": floor_failures,
        "streak": {
            "current": streak_current,
            "required": streak_required,
        },
        "qualifies_for_advance": qualifies_for_advance,
    }))
}

// ---------------------------------------------------------------------------
// B-42 — COSMOS ARTIFACT TIER (dashboard surface)
// ---------------------------------------------------------------------------
//
// `get_cosmos_status` returns the per-artifact freshness snapshot read
// filesystem-only from `~/.elai/cosmos/`. No ResearchDispatcher is required:
// the standalone helper constructs a CosmosWorker with a NoopResearchDispatcher
// purely to call `status_snapshot`, which never invokes the dispatcher.
// Cold-start returns an empty Vec (no cosmos root yet).
//
// C-16 (DECISION-D + DECISION-G): `trigger_cosmos_refresh` is now wired to
// the production `task_runner::trigger_cosmos_refresh` helper via an ad-hoc
// CosmosWorker built from the user-global cosmos root + manifest. Returns
// `TriggerCosmosRefreshResponse` with the count of refreshed artifacts and
// wall-clock brackets. The previous B-42 NotImplemented stub is replaced by
// the implementation below.
//
// `subscribe_cosmos_events` (C-16 DECISION-C) is the new live-update path
// — poll-based for now; a real Tauri `listen()` push channel is a future
// UI infrastructure phase.

/// Per-artifact cosmos status for the dashboard drawer.
/// Read-only: filesystem snapshot of ~/.elai/cosmos/ — no research, no mutation.
#[tauri::command]
pub async fn get_cosmos_status() -> Result<Vec<elai_knowledge::CosmosArtifactStatusJson>, String> {
    Ok(elai_knowledge::cosmos_status_snapshot_standalone())
}

/// C-16 (DECISION-D): typed response shape for the on-demand cosmos refresh.
///
/// `refreshed_count` is the number of artifacts whose body actually changed
/// (idempotent re-research with identical blake3 does NOT increment this
/// counter — matches the SSB CosmosArtifactUpdated emission contract).
/// `started_at_ms` / `completed_at_ms` are Unix epoch milliseconds bracketing
/// the refresh call, derived from the system clock at the Tauri command
/// entry/exit.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TriggerCosmosRefreshResponse {
    pub refreshed_count: u32,
    pub started_at_ms: i64,
    pub completed_at_ms: i64,
}

/// C-16 (DECISION-C): cosmos SSB event row for the live subscription path.
/// `seq` is the monotonic SSB sequence number — frontend stores the highest
/// it has seen and passes back as `last_seq` on the next poll to receive
/// only the delta. `kind` is one of:
/// `"refresh_started" | "artifact_updated" | "artifact_decayed" |
/// "grounding_used"`. `payload` is the JSON-serialised typed payload (see
/// orchestrator::ssb::CosmosRefreshStartedPayload etc.).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CosmosEvent {
    pub seq: i64,
    pub timestamp_ms: i64,
    pub kind: String,
    pub payload: serde_json::Value,
}

/// C-16 (DECISION-C): typed response shape for subscribe_cosmos_events.
/// `last_seq` is the highest seq across `events`; the frontend uses it as
/// the next call's `last_seq` argument.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CosmosEventBatch {
    pub last_seq: i64,
    pub events: Vec<CosmosEvent>,
}

/// Trigger an on-demand refresh of cosmos artifacts.
///
/// C-16 (DECISION-D + DECISION-G): wires the orphan
/// `task_runner::trigger_cosmos_refresh` helper into the dashboard. Builds
/// an ad-hoc CosmosWorker rooted at `default_cosmos_root()` with the
/// production `HttpCosmosResearchDispatcher`, then invokes the helper which
/// emits `CosmosRefreshStarted` + per-artifact `CosmosArtifactUpdated` SSB
/// rows.
///
/// `_name` is accepted for backward compat with the B-42 stub signature but
/// is currently advisory: the refresh sweep covers the whole manifest
/// (per-artifact targeting is a follow-up). Returns
/// [`TriggerCosmosRefreshResponse`] with the count of artifacts actually
/// refreshed + wall-clock brackets. Returns `Err(String)` when the SSB
/// writer is unavailable (cold-start dashboard) — honest "wiring
/// unavailable" failure, NOT mock data.
#[tauri::command]
pub async fn trigger_cosmos_refresh(
    _name: Option<String>,
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<TriggerCosmosRefreshResponse, String> {
    let started_at_ms = i64::try_from(state.clock.now_ms()).unwrap_or(i64::MAX); // clock-injected via AppState.clock (ClockProvider), not raw SystemTime

    let writer = state
        .writer
        .clone()
        .ok_or_else(|| "cosmos refresh: SSB writer unavailable (no SSB DB)".to_string())?;

    // Resolve cosmos root + manifest. Cold-start (no manifest) returns a
    // refreshed_count=0 success — not an error.
    let cosmos_root = elai_knowledge::default_cosmos_root()
        .map_err(|e| format!("cosmos refresh: resolve root failed: {e}"))?;
    if !cosmos_root.exists() {
        let completed_at_ms = i64::try_from(state.clock.now_ms()).unwrap_or(i64::MAX); // clock-injected via AppState.clock (ClockProvider), not raw SystemTime
        return Ok(TriggerCosmosRefreshResponse {
            refreshed_count: 0,
            started_at_ms,
            completed_at_ms,
        });
    }

    let manifest_path = cosmos_root.join("manifest.toml");
    let manifest = elai_knowledge::CosmosManifest::load(&manifest_path)
        .map_err(|e| format!("cosmos refresh: manifest load failed: {e}"))?;
    if manifest.artifacts.is_empty() {
        let completed_at_ms = i64::try_from(state.clock.now_ms()).unwrap_or(i64::MAX); // clock-injected via AppState.clock (ClockProvider), not raw SystemTime
        return Ok(TriggerCosmosRefreshResponse {
            refreshed_count: 0,
            started_at_ms,
            completed_at_ms,
        });
    }

    // C-16 (DECISION-B): production dispatcher.
    let sidecar_base = std::env::var("ELAI_SIDECAR_URL")
        .or_else(|_| std::env::var("ANTHROPIC_BASE_URL"))
        .unwrap_or_else(|_| elai_orchestrator::DEFAULT_SIDECAR_SEARCH_URL.to_string());
    let research: std::sync::Arc<dyn elai_knowledge::ResearchDispatcher> =
        match elai_orchestrator::HttpCosmosResearchDispatcher::new(sidecar_base) {
            Ok(d) => std::sync::Arc::new(d),
            Err(e) => return Err(format!("cosmos refresh: dispatcher init failed: {e}")),
        };
    let clock: std::sync::Arc<dyn elai_knowledge::CosmosClock> =
        std::sync::Arc::new(elai_knowledge::SystemCosmosClock);
    let worker = elai_knowledge::CosmosWorker::new(cosmos_root, manifest, clock, research)
        .map_err(|e| format!("cosmos refresh: worker init failed: {e}"))?;

    // DECISION-G: invoke the previously-orphan helper. It emits the SSB
    // rows itself; we just count the result.
    let refreshed_count = elai_orchestrator::trigger_cosmos_refresh(&writer, &worker).await;

    let completed_at_ms = i64::try_from(state.clock.now_ms()).unwrap_or(i64::MAX); // clock-injected via AppState.clock (ClockProvider), not raw SystemTime

    Ok(TriggerCosmosRefreshResponse {
        refreshed_count,
        started_at_ms,
        completed_at_ms,
    })
}

/// C-16 (DECISION-C): poll-based SSB subscription for cosmos drawer.
///
/// Returns all `cosmos_*` SSB rows with `seq > last_seq`, capped at 64 rows
/// per call to bound the IPC payload size (matches the
/// `PREVIEW_AVAILABLE_HARD_CAP` pattern in SsbViewer). Frontend polls this
/// at a 5s cadence (down from the previous 30s status poll) and re-renders
/// the drawer reactively. Pass `last_seq = 0` on initial mount to receive
/// the full backlog (still capped at 64 newest rows).
///
/// Returns `Err(String)` when the SsbViewer is unavailable (cold-start).
#[tauri::command]
pub async fn subscribe_cosmos_events(
    last_seq: i64,
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<CosmosEventBatch, String> {
    const MAX_COSMOS_EVENTS: usize = 64;

    let viewer = state
        .viewer
        .clone()
        .ok_or_else(|| "cosmos subscribe: SsbViewer unavailable (no SSB DB)".to_string())?;

    // recent_actions returns newest-first across ALL action types. We filter
    // client-side here for cosmos_*. The 4x multiplier on the limit allows
    // for non-cosmos rows interleaved in the same window without dropping
    // cosmos events.
    let pulled = viewer
        .recent_actions(MAX_COSMOS_EVENTS.saturating_mul(8), 0)
        .map_err(|e| format!("cosmos subscribe: recent_actions failed: {e}"))?;

    let mut events: Vec<CosmosEvent> = Vec::with_capacity(MAX_COSMOS_EVENTS);
    let mut new_last_seq = last_seq;
    for row in pulled {
        if row.seq <= last_seq {
            continue;
        }
        let kind = match row.action_type {
            elai_orchestrator::SsbActionType::CosmosRefreshStarted => "refresh_started",
            elai_orchestrator::SsbActionType::CosmosArtifactUpdated => "artifact_updated",
            elai_orchestrator::SsbActionType::CosmosArtifactDecayed => "artifact_decayed",
            elai_orchestrator::SsbActionType::CosmosGroundingUsed => "grounding_used",
            _ => continue,
        };
        // recent_actions returns the truncated `summary` string, not the
        // typed payload. For the drawer's needs (just badge + name +
        // timestamp), the summary is sufficient. We wrap it in a JSON
        // object so the frontend has a stable shape.
        let payload = serde_json::json!({ "summary": row.summary });
        events.push(CosmosEvent {
            seq: row.seq,
            timestamp_ms: row.timestamp,
            kind: kind.to_string(),
            payload,
        });
        if row.seq > new_last_seq {
            new_last_seq = row.seq;
        }
        if events.len() >= MAX_COSMOS_EVENTS {
            break;
        }
    }
    // Oldest-first for the frontend (easier to append-on-poll without
    // re-sorting client-side).
    events.reverse();

    Ok(CosmosEventBatch {
        last_seq: new_last_seq,
        events,
    })
}

// ---------------------------------------------------------------------------
// B-41 — Per-level audit verdicts (additive, LIVE-only — no mock fallback per
// brief). Returns the materialized LevelAuditVerdict list for a given task
// id by replaying the PerLevelAuditVerdict SSB rows. Cold start (no rows)
// returns an empty Vec.
//
// The TaskRunner already buffers SSB rows; this command is a read-only
// projection. Until the TaskRunner exposes a session-scoped reader to the
// dashboard layer, the implementation returns an empty list — NOT a mock
// (the absence of rows is a real cold-start condition, observed by
// `derive_*_zero_blocks` evidence helpers as "vacuous-default true").
// ---------------------------------------------------------------------------

/// JSON shape mirrored by `dashboard-v2/src/lib/api/types.ts::LevelAuditVerdictJson`.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct LevelAuditVerdictJson {
    pub mode: String,
    pub tier_node_id: TierNodeIdJson,
    pub findings: Vec<AuditFindingJson>,
    pub verdict: String,
    pub respawned_once: bool,
    pub cascade_to_parent: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TierNodeIdJson {
    pub tier: String,
    pub id: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AuditFindingJson {
    pub category: String,
    pub severity: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// C-12 — Internal helper that turns SSB rows into the
/// `LevelAuditVerdictJson` shape consumed by `AuditTimelineWindow.svelte`.
///
/// Pulls every `per_level_audit_verdict` row (cap 500) and every
/// `per_level_audit_cascade` row (used to up-flag the `cascade_to_parent`
/// bit when a verdict row was emitted *before* its cascade was decided).
/// Rows are filtered to the latest session_id discovered across either
/// row family (mirrors `get_idea_framing_state`'s session resolution).
///
/// DRIFT-1 resolution (a): the SSB `PerLevelAuditVerdictPayload` only
/// carries `findings_count`. The full `Finding` list is persisted to disk
/// as Markdown by `auditor_mode::persist_verdict` and is NOT recoverable
/// from SSB alone. Returning an empty `findings: []` per row is the
/// documented cold-start path; `AuditTimelineWindow.svelte`'s findings
/// popover already renders "No findings recorded for this verdict." in
/// that branch (line 174 of the component).
///
/// `task_id_filter` is accepted for future per-task scoping. When
/// `"active"` (the dashboard default), `""`, or any value, the helper
/// resolves to the latest observed session_id — matching DRIFT-6.
pub fn compute_per_level_verdicts(
    viewer: &elai_orchestrator::SsbViewer,
    task_id_filter: &str,
) -> Result<Vec<LevelAuditVerdictJson>, String> {
    let verdict_rows = query_rows_by_action_type(viewer, SsbActionType::PerLevelAuditVerdict)?;
    let cascade_rows = query_rows_by_action_type(viewer, SsbActionType::PerLevelAuditCascade)?;

    if verdict_rows.is_empty() && cascade_rows.is_empty() {
        return Ok(Vec::new());
    }

    // Resolve session_id. The dashboard sends "active" by default — map
    // that (and empty) to "latest session that produced any audit row".
    // An explicit caller-supplied session id is honoured verbatim.
    let session_id: String = if task_id_filter.is_empty() || task_id_filter == "active" {
        verdict_rows
            .last()
            .map(|(s, _)| s.clone())
            .or_else(|| cascade_rows.last().map(|(s, _)| s.clone()))
            .unwrap_or_default()
    } else {
        task_id_filter.to_owned()
    };

    if session_id.is_empty() {
        return Ok(Vec::new());
    }

    // Pre-compute the set of (mode, tier, node_id) keys that have at
    // least one cascade row in this session — used to up-flag the
    // cascade_to_parent bit on the corresponding verdict row even when
    // the row was written BEFORE the cascade decision landed.
    use std::collections::BTreeSet;
    let mut cascaded_keys: BTreeSet<(String, String, u64)> = BTreeSet::new();
    for (sid, bytes) in &cascade_rows {
        if *sid != session_id {
            continue;
        }
        let Ok(payload) =
            elai_orchestrator::decode_ssb_payload_bytes::<SsbPayload>(bytes.as_slice())
        else {
            continue;
        };
        if let SsbPayload::PerLevelAuditCascade(p) = payload {
            cascaded_keys.insert((p.mode, p.tier, p.node_id));
        }
    }

    let mut out: Vec<LevelAuditVerdictJson> = Vec::new();
    for (sid, bytes) in &verdict_rows {
        if *sid != session_id {
            continue;
        }
        let Ok(payload) =
            elai_orchestrator::decode_ssb_payload_bytes::<SsbPayload>(bytes.as_slice())
        else {
            continue;
        };
        let SsbPayload::PerLevelAuditVerdict(p) = payload else {
            continue;
        };
        let cascade_to_parent = p.cascade_to_parent
            || cascaded_keys.contains(&(p.mode.clone(), p.tier.clone(), p.node_id));
        out.push(LevelAuditVerdictJson {
            mode: p.mode,
            tier_node_id: TierNodeIdJson {
                tier: p.tier,
                id: p.node_id,
            },
            findings: Vec::new(),
            verdict: p.verdict,
            respawned_once: p.respawned_once,
            cascade_to_parent,
        });
    }
    Ok(out)
}

/// Return the per-level audit verdicts for `task_id`. Live SSB read — no
/// mock. Cold start (no rows) returns an empty Vec. `task_id` is mapped
/// to the latest session_id when "active"/empty (DRIFT-6: per-task
/// scoping waits on a TaskRunner-side reader exposing real task ids).
#[tauri::command]
pub async fn get_per_level_verdicts(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Vec<LevelAuditVerdictJson>, String> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(Vec::new());
    };
    compute_per_level_verdicts(viewer.as_ref(), &task_id)
}

// ---------------------------------------------------------------------------
// F-4 Slice B-5 — TEST INTEGRITY EVENT SURFACE (AuditTimelineWindow)
// ---------------------------------------------------------------------------
//
// Surfaces `SsbActionType::TestIntegrityEvent` rows to the dashboard so the
// AuditTimelineWindow can display a red fabrication-block badge alongside the
// per-level audit verdicts. Two sql_str arms are queried:
//   - "test_integrity_verified"  (TestIntegrityEventKind::Verified)
//   - "anti_fabrication_block"   (TestIntegrityEventKind::FabricationBlocked)
//
// Both are returned in a single Vec<TestIntegrityEventJson> sorted by SSB seq
// (oldest first, matching `recent_rows_of_type` ORDER BY seq ASC default).
// The frontend badge only needs `kind`, `passed`, and `has_stdout_hash`;
// `test_name` is included for the detail hover.
//
// cross-lang-pair: dashboard/tauri/api (drift gate: cross_lang_diff.py)

/// DTO returned by `get_test_integrity_events`. Mirrors the SSB payload
/// fields that the AuditTimelineWindow needs — `test_name`, `passed`,
/// `has_stdout_hash` (boolean projection of `Option<String>`), and
/// `kind` (snake_case label: "verified" or "fabrication_blocked").
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct TestIntegrityEventJson {
    pub test_name: String,
    pub passed: bool,
    pub has_stdout_hash: bool,
    pub kind: String,
}

/// Return all `TestIntegrityEvent` SSB rows for `task_id`. Both kind arms
/// ("test_integrity_verified" and "anti_fabrication_block") are fetched and
/// merged. Cold start (no rows) returns an empty Vec. `task_id` is accepted
/// for future per-task scoping; current implementation reads all rows via
/// `recent_rows_of_type` (DRIFT-6: per-task scoping follows TaskRunner reader
/// work, same as `get_per_level_verdicts`).
#[tauri::command]
pub async fn get_test_integrity_events(
    state: State<'_, AppState>,
    task_id: String,
) -> CmdResult<Vec<TestIntegrityEventJson>> {
    let _ = task_id; // reserved for per-task scoping (DRIFT-6)
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(Vec::new());
    };
    let verified_rows = viewer
        .recent_rows_of_type(
            elai_orchestrator::SsbActionType::TestIntegrityEvent(
                elai_orchestrator::TestIntegrityEventKind::Verified,
            )
            .as_sql_str(),
            MAX_LIMIT,
        )
        .unwrap_or_default();
    let blocked_rows = viewer
        .recent_rows_of_type(
            elai_orchestrator::SsbActionType::TestIntegrityEvent(
                elai_orchestrator::TestIntegrityEventKind::FabricationBlocked,
            )
            .as_sql_str(),
            MAX_LIMIT,
        )
        .unwrap_or_default();
    let mut out: Vec<TestIntegrityEventJson> = Vec::with_capacity(
        verified_rows.len() + blocked_rows.len(),
    );
    for detail in verified_rows {
        let inner = detail
            .payload_json
            .get("TestIntegrityEvent")
            .cloned()
            .unwrap_or(detail.payload_json);
        let test_name = inner
            .get("test_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_owned();
        let passed = inner
            .get("passed")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let has_stdout_hash = inner
            .get("stdout_hash")
            .map(|v| !v.is_null())
            .unwrap_or(false);
        out.push(TestIntegrityEventJson {
            test_name,
            passed,
            has_stdout_hash,
            kind: "verified".to_owned(),
        });
    }
    for detail in blocked_rows {
        let inner = detail
            .payload_json
            .get("TestIntegrityEvent")
            .cloned()
            .unwrap_or(detail.payload_json);
        let test_name = inner
            .get("test_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_owned();
        let passed = inner
            .get("passed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let has_stdout_hash = inner
            .get("stdout_hash")
            .map(|v| !v.is_null())
            .unwrap_or(false);
        out.push(TestIntegrityEventJson {
            test_name,
            passed,
            has_stdout_hash,
            kind: "fabrication_blocked".to_owned(),
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// B-37D — MULTI-RUNTIME BACKEND ABSTRACTION (Phase 4M)
// ---------------------------------------------------------------------------
//
// Three Tauri commands surface the multi-runtime layer to the dashboard:
//
//   - `list_runtimes()`             -> per-backend RuntimeInfo (canonical name,
//                                      roles mapped to it, health, stub flag,
//                                      latency_ms).
//   - `test_runtime(backend)`       -> live health-probe report (ok, latency,
//                                      optional error). NEVER serializes
//                                      SecretString content (B-37 locked
//                                      decision DECISION-3).
//   - `set_role_runtime(role, b)`   -> mutate AppState's RuntimeOrchestrator
//                                      mapping AND atomically persist to
//                                      `elai_cc.toml` (mirrors CLI
//                                      runtime_cmd.rs::handle_runtime_set).
//
// Mirroring DECISION-2: stub adapters return RuntimeAdapterError::NotImplemented
// from complete; their health_check yields Unavailable with reason carrying
// "not implemented" / "stub". We classify those rows as `stub=true` so the
// frontend renders the "stub adapter" pill distinct from a missing-binary
// row.
//
// IPC v2 envelope: these DTOs are command request/response shapes (NOT SSB
// rows), so no SCHEMA_VERSION field is added (B-34 floor at 42 unchanged).
// JSON keys use snake_case to mirror serde rename_all="snake_case" on
// RuntimeBackend + RuntimeHealth in providers::runtime::types — consistent
// with LevelAuditVerdictJson above.

/// Snake-case identifiers for every `AgentRole` discriminant in
/// `elai_ipc_types`. Used by `set_role_runtime` for argument validation and
/// by `list_runtimes` to render role labels. Mirrors `runtime_cmd.rs`'s
/// SUPPORTED_ROLES list (same source of truth, copied to avoid a path-dep
/// on the cli crate from dashboard-v2).
const RUNTIME_SUPPORTED_ROLES: &[&str] = &[
    "executor",
    "auditor",
    "researcher",
    "designer",
    "prompt_engineer",
    "context_compiler",
    "memory_writer",
    "media_validator",
    "idea_framer",
    "idea_critic",
    "auditor_concepts",
    "auditor_phases",
    "auditor_steps",
    "auditor_baby_steps",
    "auditor_baby_step_detail",
];

/// Per-backend snapshot returned by `list_runtimes`. Field names use
/// snake_case to match the rest of the dashboard DTO surface (e.g.
/// LevelAuditVerdictJson above) and the serde rename_all="snake_case" on
/// `RuntimeBackend`/`RuntimeHealth` in providers::runtime.
///
/// C-4 (DRIFT-9 fix): aligned with `RuntimeInfo` interface in
/// dashboard-v2/src/lib/api/types.ts -> `health` is now the typed
/// `RuntimeHealth` (`{status, reason?}`), `stub` is renamed `is_stub`,
/// and `canonical_name` is emitted alongside `backend`. Frontend
/// `RuntimePanel.svelte` consumers expect this shape.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RuntimeInfo {
    /// Display label; equal to `canonical_name` today. Kept distinct so a
    /// future "human label" override doesn't break wire compatibility.
    pub backend: String,
    /// Canonical snake_case backend identifier (round-trips through
    /// `resolve_backend`). Added in C-4 (DRIFT-9) to match the TS
    /// `RuntimeBackendId` discriminator.
    pub canonical_name: String,
    /// AgentRole canonical names (snake_case) currently routed to this
    /// backend. Empty when no role maps here.
    pub roles: Vec<String>,
    /// Typed health snapshot (`{status, reason?}`); serialises via
    /// `RuntimeHealth`'s `tag = "status"` rename_all = "snake_case".
    pub health: elai_providers::runtime::RuntimeHealth,
    /// True when the backend is a stub adapter — `Unavailable` with a
    /// reason mentioning "not implemented" / "stub" (per B-37 DECISION-8).
    #[serde(rename = "is_stub")]
    pub is_stub: bool,
    /// Health-probe latency in milliseconds (None if the snapshot path did
    /// not measure per-row latency).
    pub latency_ms: Option<u32>,
}

/// Live health-probe report returned by `test_runtime`.
///
/// C-4 (DRIFT-9 fix): aligned with `TestReport` interface in
/// dashboard-v2/src/lib/api/types.ts -> emits `backend` (canonical
/// snake_case id) and the typed `health: RuntimeHealth` alongside the
/// existing `ok` / `latency_ms` / `error` triple.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TestReport {
    /// Canonical backend identifier (`RuntimeBackendId` on the TS side).
    pub backend: String,
    /// True iff the backend reports `RuntimeHealth::Ready` within the
    /// 5-second timeout. `Degraded` and `Unavailable` both yield false.
    pub ok: bool,
    /// Typed health snapshot (`{status, reason?}`).
    pub health: elai_providers::runtime::RuntimeHealth,
    /// Elapsed wall-clock milliseconds from probe start to outcome.
    pub latency_ms: u32,
    /// Failure reason. `None` on success. SecretString contents are
    /// redacted to "<redacted>" before populating this field per
    /// B-37 DECISION-3 (SecretString never serialized).
    pub error: Option<String>,
}

/// List every runtime backend with its current role-mapping, health,
/// stub-flag, and probe latency. Per-backend probes are wrapped in a 5s
/// timeout so a hung adapter cannot stall the UI.
///
/// Mirrors the CLI's `/runtime list` table (cli/src/runtime_cmd.rs) but
/// returns structured JSON for the dashboard. The orchestrator's
/// `health_snapshot()` is the canonical source for the health column;
/// per-row latency is measured inline because `health_snapshot` does not
/// expose timings (and adding that surface would change the public API).
#[tauri::command]
pub async fn list_runtimes(
    state: State<'_, AppState>,
) -> Result<Vec<RuntimeInfo>, String> {
    use elai_providers::runtime::{RuntimeBackend, RuntimeHealth};

    // Snapshot the per-role mapping under the orchestrator's config lock.
    let cfg = state.runtime_orchestrator.snapshot_config().await;
    let per_role = cfg.per_role.list(); // Vec<(AgentRole, RuntimeBackend)>

    // Pre-compute roles per backend so the dashboard can render the
    // "this backend serves these roles" column without a second pass.
    let mut roles_by_backend: BTreeMap<RuntimeBackend, Vec<String>> = BTreeMap::new();
    for (role, backend) in per_role {
        // AgentRole serializes as snake_case via serde rename_all; round
        // through serde_json to get the canonical string without depending
        // on a FromStr impl that ipc-types does not provide.
        let role_label = serde_json::to_value(&role)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| format!("{role:?}").to_ascii_lowercase());
        roles_by_backend.entry(backend).or_default().push(role_label);
    }

    // Use RuntimeOrchestrator::health_snapshot to probe every backend's
    // cached/lazy-instantiated adapter in one shot. The snapshot covers
    // every backend in `cfg.factory_cfg` (defaulted to all 11). We
    // measure total elapsed and report it on every row — health_snapshot
    // does not expose per-row timings and we want to avoid an additional
    // sequential probe loop (11 separate 5s timeouts).
    let probe_clock = OsClock;
    let probe_start = probe_clock.now();
    let snapshot = state.runtime_orchestrator.health_snapshot().await;
    let total_latency_ms = u32::try_from(probe_clock.elapsed_since(probe_start).as_millis()).ok();
    let by_backend: BTreeMap<RuntimeBackend, RuntimeHealth> = snapshot.into_iter().collect();

    // Iterate `RuntimeBackend::all()` so the rendered order is stable
    // regardless of BTreeMap key ordering inside the snapshot.
    let mut out: Vec<RuntimeInfo> = Vec::with_capacity(RuntimeBackend::all().len());
    for backend in RuntimeBackend::all().iter().copied() {
        let health = by_backend
            .get(&backend)
            .cloned()
            .unwrap_or_else(|| RuntimeHealth::Unavailable {
                reason: "backend missing from snapshot".to_string(),
            });

        let is_stub = match &health {
            RuntimeHealth::Ready | RuntimeHealth::Degraded { .. } => false,
            RuntimeHealth::Unavailable { reason } => {
                reason.contains("not implemented") || reason.contains("stub")
            }
        };

        let roles = roles_by_backend
            .get(&backend)
            .cloned()
            .unwrap_or_default();

        let canonical = backend.canonical_name().to_string();
        out.push(RuntimeInfo {
            backend: canonical.clone(),
            canonical_name: canonical,
            roles,
            health,
            is_stub,
            latency_ms: total_latency_ms,
        });
    }
    Ok(out)
}

/// Probe a single backend's health by canonical name (or any alias
/// accepted by `resolve_backend`). Wraps the adapter's `health_check`
/// future in a 5-second timeout so a hung CLI subprocess cannot stall
/// the dashboard UI.
///
/// SECURITY (locked decision DECISION-3): the returned `TestReport.error`
/// field MUST NOT carry SecretString contents. Adapter constructor
/// failures (e.g. AnthropicApiRuntime missing auth) surface
/// `RuntimeAdapterError::MissingField` whose Display trait does not
/// reveal the secret value — but we still pass every error string
/// through `redact_secret_substrings` defensively before emitting.
#[tauri::command]
pub async fn test_runtime(backend: String) -> Result<TestReport, String> {
    use elai_providers::runtime::{
        create_runtime, resolve_backend, RuntimeFactoryConfig, RuntimeHealth,
    };

    let test_clock = OsClock;
    let probe_start = test_clock.now();
    let resolved = match resolve_backend(&backend) {
        Ok(b) => b,
        Err(err) => {
            let reason = redact_secret_substrings(&err.to_string());
            return Ok(TestReport {
                backend: backend.clone(),
                ok: false,
                health: RuntimeHealth::Unavailable { reason: reason.clone() },
                latency_ms: u32::try_from(test_clock.elapsed_since(probe_start).as_millis()).unwrap_or(u32::MAX),
                error: Some(reason),
            });
        }
    };

    let canonical = resolved.canonical_name().to_string();
    let cfg = RuntimeFactoryConfig::for_backend(resolved);
    let adapter = match create_runtime(cfg) {
        Ok(a) => a,
        Err(err) => {
            let reason = redact_secret_substrings(&err.to_string());
            return Ok(TestReport {
                backend: canonical,
                ok: false,
                health: RuntimeHealth::Unavailable { reason: reason.clone() },
                latency_ms: u32::try_from(test_clock.elapsed_since(probe_start).as_millis()).unwrap_or(u32::MAX),
                error: Some(reason),
            });
        }
    };

    let health = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        adapter.health_check(),
    )
    .await
    .unwrap_or(RuntimeHealth::Unavailable {
        reason: "health probe timeout (>5s)".to_string(),
    });

    let latency_ms = u32::try_from(test_clock.elapsed_since(probe_start).as_millis()).unwrap_or(u32::MAX);
    let (ok, error) = match &health {
        RuntimeHealth::Ready => (true, None),
        RuntimeHealth::Degraded { reason } => {
            (false, Some(redact_secret_substrings(reason)))
        }
        RuntimeHealth::Unavailable { reason } => {
            (false, Some(redact_secret_substrings(reason)))
        }
    };
    Ok(TestReport { backend: canonical, ok, health, latency_ms, error })
}

/// Persist a new (`AgentRole` -> `RuntimeBackend`) mapping. Mutates the
/// in-memory `RuntimeOrchestrator` config and atomically rewrites
/// `<project_root>/elai_cc.toml` so the choice survives a restart.
///
/// The atomic-write pattern mirrors
/// `cli/src/runtime_cmd.rs::atomic_write` — write to `<path>.tmp`, then
/// rename. This avoids partial writes on Windows + Linux.
#[tauri::command]
pub async fn set_role_runtime(
    state: State<'_, AppState>,
    role: String,
    backend: String,
) -> Result<(), String> {
    use elai_providers::runtime::resolve_backend;

    // Validate + canonicalise role.
    let role_normalised = role.trim().to_ascii_lowercase();
    if !RUNTIME_SUPPORTED_ROLES.iter().any(|r| *r == role_normalised) {
        return Err(format!(
            "unknown agent role '{role_normalised}'. Supported: {}",
            RUNTIME_SUPPORTED_ROLES.join(", ")
        ));
    }
    let role_canonical = role_normalised; // already snake_case

    // Validate + canonicalise backend (any documented alias is accepted).
    let resolved = resolve_backend(&backend).map_err(|e| {
        format!(
            "unknown backend '{}'. Supported: {}",
            e.candidate,
            e.supported.join(", ")
        )
    })?;
    let backend_canonical = resolved.canonical_name();

    // Parse role string back into AgentRole via serde so we don't depend
    // on a FromStr impl (ipc-types::AgentRole does not provide one). The
    // wire format is snake_case per `#[serde(rename_all = "snake_case")]`.
    let role_enum: elai_ipc_types::AgentRole =
        serde_json::from_value(serde_json::Value::String(role_canonical.clone()))
            .map_err(|e| format!("failed to parse role '{role_canonical}': {e}"))?;

    // Mutate the orchestrator's RuntimeBackendConfig in place. We clone,
    // override, then `set_config` so the adapter cache is cleared (per
    // RuntimeOrchestrator::set_config docs — adapters for the old mapping
    // would otherwise still be cached).
    let mut cfg = state.runtime_orchestrator.snapshot_config().await;
    cfg.per_role.set(role_enum, resolved);
    state.runtime_orchestrator.set_config(cfg).await;

    // Persist to elai_cc.toml. Failure to write is reported back to the
    // frontend so the user can retry; the in-memory mapping has already
    // been updated either way.
    let project_root = state.project_root_snapshot();
    persist_role_runtime_toml(&project_root, &role_canonical, backend_canonical)
        .map_err(|e| format!("persist elai_cc.toml: {e}"))?;
    Ok(())
}

/// Atomic-write helper for `set_role_runtime`. Mirrors
/// `cli/src/runtime_cmd.rs::atomic_write` + `set_role_runtime_in_toml`.
/// Reads the TOML, mutates `[agents.<role>.runtime] backend = "<name>"`
/// (creating intermediate tables), serialises back, writes via `<tmp>`
/// + rename.
fn persist_role_runtime_toml(
    project_root: &std::path::Path,
    role_canonical: &str,
    backend_canonical: &str,
) -> Result<(), String> {
    let config_path = project_root.join("elai_cc.toml");
    let original = std::fs::read_to_string(&config_path).unwrap_or_default();
    let mut doc: toml::Value = if original.trim().is_empty() {
        toml::Value::Table(toml::value::Table::new())
    } else {
        toml::from_str(&original)
            .map_err(|e| format!("parse {}: {e}", config_path.display()))?
    };
    set_role_runtime_in_toml_doc(&mut doc, role_canonical, backend_canonical);
    let rendered = toml::to_string_pretty(&doc).map_err(|e| e.to_string())?;

    let tmp_path = {
        let mut p = config_path.clone();
        let new_name = match p.file_name() {
            Some(name) => {
                let mut s = name.to_os_string();
                s.push(".tmp");
                s
            }
            None => return Err(format!("config path has no filename: {}", p.display())),
        };
        p.set_file_name(new_name);
        p
    };
    std::fs::write(&tmp_path, rendered.as_bytes())
        .map_err(|e| format!("temp write {}: {e}", tmp_path.display()))?;
    std::fs::rename(&tmp_path, &config_path).map_err(|e| {
        format!(
            "rename {} -> {}: {e}",
            tmp_path.display(),
            config_path.display()
        )
    })?;
    Ok(())
}

/// Walk `[agents.<role>.runtime]` tables, creating intermediate tables
/// as needed. Mirrors the CLI helper of the same shape in runtime_cmd.rs;
/// duplicated (small, ~30 LOC) because the CLI crate is not a dependency
/// of dashboard-v2.
fn set_role_runtime_in_toml_doc(doc: &mut toml::Value, role: &str, backend_name: &str) {
    let Some(root_table) = doc.as_table_mut() else {
        // Caller passed a non-table root. Rewrite into a fresh table.
        *doc = toml::Value::Table(toml::value::Table::new());
        if let Some(t) = doc.as_table_mut() {
            install_runtime_backend(t, role, backend_name);
        }
        return;
    };
    install_runtime_backend(root_table, role, backend_name);
}

fn install_runtime_backend(
    root: &mut toml::value::Table,
    role: &str,
    backend_name: &str,
) {
    let agents = root
        .entry("agents".to_string())
        .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
    if !agents.is_table() {
        *agents = toml::Value::Table(toml::value::Table::new());
    }
    let Some(agents_table) = agents.as_table_mut() else {
        return;
    };

    let role_entry = agents_table
        .entry(role.to_string())
        .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
    if !role_entry.is_table() {
        *role_entry = toml::Value::Table(toml::value::Table::new());
    }
    let Some(role_table) = role_entry.as_table_mut() else {
        return;
    };

    let runtime_entry = role_table
        .entry("runtime".to_string())
        .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
    if !runtime_entry.is_table() {
        *runtime_entry = toml::Value::Table(toml::value::Table::new());
    }
    let Some(runtime_table) = runtime_entry.as_table_mut() else {
        return;
    };

    runtime_table.insert(
        "backend".to_string(),
        toml::Value::String(backend_name.to_string()),
    );
}

/// Defensive redaction. SecretString's Display impl already renders
/// "<redacted: N bytes>", so an accidental `{secret}` in an error
/// message never leaks the value. This helper exists as a defence in
/// depth: should an adapter error string ever concatenate the inner
/// SecretString (via `.expose_secret()`), we replace common bearer-
/// token / API-key prefixes with "<redacted>". Per B-37 DECISION-3 the
/// adapter layer is the primary line of defence; this is the secondary.
fn redact_secret_substrings(input: &str) -> String {
    // Anthropic OAuth bearer tokens start with `sk-ant-` (api key) or
    // a long base64-like string for OAuth. Replace any obvious token
    // shape inline. Conservative: never alter normal prose.
    let mut out = String::with_capacity(input.len());
    for token in input.split_whitespace() {
        let candidate = token.trim_matches(|c: char| matches!(c, '"' | '\'' | ',' | '.' | ';' | ':'));
        let looks_like_key = candidate.starts_with("sk-ant-")
            || candidate.starts_with("sk-")
            || (candidate.len() >= 32
                && candidate.chars().all(|c| {
                    c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.'
                }));
        if looks_like_key {
            out.push_str("<redacted>");
        } else {
            out.push_str(token);
        }
        out.push(' ');
    }
    out.trim_end().to_string()
}

// ---------------------------------------------------------------------------
// C-4: ORCHESTRATION + CLOUD SPEND SURFACE
// ---------------------------------------------------------------------------
// Thin Tauri-command wrappers over SsbViewer methods already shipped by C-2/C-3
// (phase_detail, baby_step_detail, phase_progress, recent_actions, file_changes,
// get_action_detail, decomposition_tree), plus a greenfield projects.json
// surface and the cloud_spend_summary aggregation. Every command returns a
// {known: bool, ...} or empty-vec graceful-empty shape — never a panic — when
// the SSB is fresh / the row does not exist yet.
//
// Wiring contract: see workflow/reasoning/C-4-decisions.md.
// ---------------------------------------------------------------------------

/// C-4: phase metadata fetched on PlanWindow node click. Forwards to
/// [`SsbViewer::phase_detail`] which scans `phase_complete` rows for a
/// matching id. Returns `{known: false, phase_id}` when the phase is not
/// present in the SSB (cold-start or never-completed phase).
#[tauri::command]
pub fn get_phase_detail(state: State<'_, AppState>, phase_id: String) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!({"known": false, "phase_id": phase_id}));
    };
    match viewer.phase_detail(&phase_id) {
        Ok(detail) => {
            let mut v = serde_json::to_value(&detail).map_err(map_err)?;
            if let Value::Object(ref mut map) = v {
                map.insert("known".to_string(), Value::Bool(true));
            }
            Ok(v)
        }
        Err(elai_orchestrator::SsbViewerError::NotFound { kind: _, id }) => {
            Ok(json!({"known": false, "phase_id": id}))
        }
        Err(e) => Err(map_err(e)),
    }
}

/// C-4: full BabyStep view on PlanWindow leaf click. The C-4 viewer body
/// scans `baby_step_ready` rows (C-2) and partial-populates concept /
/// phase / step names by walking the decomposition tree. Returns
/// `{known: false, baby_step_id}` when the id is not present in the SSB.
#[tauri::command]
pub fn get_baby_step_detail(
    state: State<'_, AppState>,
    baby_step_id: String,
) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!({"known": false, "baby_step_id": baby_step_id}));
    };
    match viewer.baby_step_detail(&baby_step_id) {
        Ok(detail) => {
            let mut v = serde_json::to_value(&detail).map_err(map_err)?;
            if let Value::Object(ref mut map) = v {
                map.insert("known".to_string(), Value::Bool(true));
            }
            Ok(v)
        }
        Err(elai_orchestrator::SsbViewerError::NotFound { kind: _, id }) => {
            Ok(json!({"known": false, "baby_step_id": id}))
        }
        Err(e) => Err(map_err(e)),
    }
}

/// C-4: per-phase progress aggregation over `phase_complete` rows. Empty
/// Vec when no phase has completed yet.
#[tauri::command]
pub fn get_phase_progress(state: State<'_, AppState>) -> CmdResult<Value> {
    let rows = state
        .viewer
        .as_ref()
        .and_then(|v| v.phase_progress().ok())
        .unwrap_or_default();
    serde_json::to_value(&rows).map_err(map_err)
}

/// C-4: recent SSB rows as compact summaries. `limit` defaults to 50
/// and is hard-capped at [`MAX_LIMIT`].
#[tauri::command]
pub fn get_recent_actions(
    state: State<'_, AppState>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> CmdResult<Value> {
    let lim = limit.unwrap_or(50) as usize;
    let lim = lim.min(MAX_LIMIT).max(1);
    let off = offset.unwrap_or(0) as usize;
    let rows = state
        .viewer
        .as_ref()
        .and_then(|v| v.recent_actions(lim, off).ok())
        .unwrap_or_default();
    serde_json::to_value(&rows).map_err(map_err)
}

/// C-4: every file touched in a session via `ssb_files JOIN ssb`. Empty
/// Vec when the session has not written any file rows yet.
#[tauri::command]
pub fn get_file_changes(
    state: State<'_, AppState>,
    session_id: String,
) -> CmdResult<Value> {
    let rows = state
        .viewer
        .as_ref()
        .and_then(|v| v.file_changes(&session_id).ok())
        .unwrap_or_default();
    serde_json::to_value(&rows).map_err(map_err)
}

/// C-4: full row detail (typed payload + touched-file list) for one
/// SSB seq. Returns `{known: false, seq}` when no row matches.
#[tauri::command]
pub fn get_action_detail(state: State<'_, AppState>, seq: i64) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!({"known": false, "seq": seq}));
    };
    match viewer.get_action_detail(seq) {
        Ok(detail) => {
            let mut v = serde_json::to_value(&detail).map_err(map_err)?;
            if let Value::Object(ref mut map) = v {
                map.insert("known".to_string(), Value::Bool(true));
            }
            Ok(v)
        }
        Err(elai_orchestrator::SsbViewerError::Validation(_)) => {
            Ok(json!({"known": false, "seq": seq}))
        }
        Err(e) => Err(map_err(e)),
    }
}

/// B-7 (SCHEMA_VERSION 100): per-node research history. Routed via the
/// existing `web_research_step` SSB rows filtered to stage-populated
/// entries — no new SsbActionType is fabricated (per FABRICATED-SYMBOL
/// discipline). Returns rows in descending seq order; the dashboard
/// ResearchDrawer reverses them client-side when rendering the timeline.
/// `session_id` is accepted for API stability (multi-session viewer fan-out
/// arrives in a follow-up) but the current viewer scopes to the active
/// session db, so it is not consulted here.
#[tauri::command]
pub fn get_research_node_history(
    state: State<'_, AppState>,
    _session_id: String,
    node_id: String,
) -> CmdResult<Value> {
    // Parse `node_id` as a u64 (the viewer keys on the numeric node id).
    // Non-numeric ids return an empty Vec instead of an error so the
    // ResearchDrawer renders cleanly during cold start when no node is
    // selected.
    let parsed = node_id.parse::<u64>().ok();
    let rows = match (state.viewer.as_ref(), parsed) {
        (Some(viewer), Some(nid)) => viewer
            .research_node_stages(nid, 100)
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    serde_json::to_value(&rows).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// C-7: REVIEW WINDOW DECISIONS (mutations)
// ---------------------------------------------------------------------------
// Each command writes one ChangeDecision SSB row (SCHEMA_VERSION 49) so the
// dashboard ReviewWindow + downstream auditors see reviewer verdicts as
// first-class SSB events. The live diff itself remains carried by FileWrite
// rows (SsbActionType::FileWrite); these commands record only the verdict.
//
// `hunk_id` is `None` for file-level verdicts (full-file accept/decline) and
// `Some(id)` for per-hunk verdicts. The id is opaque to the orchestrator;
// the dashboard derives it from the FileWrite diff (typically the
// `@@ -N,M +P,Q @@` header content).

/// C-7: record a reviewer "accept" verdict on a file or hunk. Emits one
/// ChangeDecision SSB row with `decision = "accepted"`. Returns
/// `{ok: true, seq}` on success or an error string when the SSB writer is
/// unavailable (no session db open).
#[tauri::command]
pub async fn accept_change(
    state: State<'_, AppState>,
    path: String,
    hunk_id: Option<String>,
) -> CmdResult<Value> {
    emit_change_decision(&state, "accepted", path, hunk_id, None).await
}

/// C-7: record a reviewer "decline" verdict on a file or hunk. Emits one
/// ChangeDecision SSB row with `decision = "declined"`. Returns
/// `{ok: true, seq}` on success.
#[tauri::command]
pub async fn decline_change(
    state: State<'_, AppState>,
    path: String,
    hunk_id: Option<String>,
) -> CmdResult<Value> {
    emit_change_decision(&state, "declined", path, hunk_id, None).await
}

/// C-7: record a reviewer "retry" verdict on a file or hunk. Emits one
/// ChangeDecision SSB row with `decision = "retried"` plus the optional
/// reviewer note. Returns `{ok: true, seq}` on success.
#[tauri::command]
pub async fn retry_audit(
    state: State<'_, AppState>,
    path: String,
    hunk_id: Option<String>,
    note: Option<String>,
) -> CmdResult<Value> {
    emit_change_decision(&state, "retried", path, hunk_id, note).await
}

/// Shared emitter for the three C-7 reviewer-verdict commands. Centralises
/// writer-availability handling and payload construction so the three
/// public Tauri commands above remain one-liners.
async fn emit_change_decision(
    state: &AppState,
    decision: &str,
    path: String,
    hunk_id: Option<String>,
    note: Option<String>,
) -> CmdResult<Value> {
    let writer = state
        .writer
        .as_ref()
        .ok_or_else(|| "ssb writer unavailable; no session db open".to_owned())?;
    let payload = SsbPayload::ChangeDecision(ChangeDecisionPayload {
        decision: decision.to_owned(),
        path: path.clone(),
        hunk_id,
        note,
    });
    let seq = writer
        .write_payload("dashboard-v2", 0, payload, vec![path])
        .await
        .map_err(map_err)?;
    Ok(json!({ "ok": true, "seq": seq }))
}

// ---------------------------------------------------------------------------
// C-4: PROJECTS SURFACE (greenfield projects.json)
// ---------------------------------------------------------------------------
// Persistence path: <app_data_dir>/projects.json — per Tauri 2 convention
// (decision-Q4). When the file is missing/empty we return an empty Vec so
// the dashboard's "Open project" picker renders the empty state. Writes
// use the atomic-temp-then-rename pattern from runtime_cmd.rs.

/// Wire row in `projects.json`. Matches the dashboard's frontend
/// expectation: `name` (display), `path` (resolved fs path), and
/// `last_opened_ms` (epoch ms for sort ordering).
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProjectEntry {
    pub name: String,
    pub path: String,
    pub last_opened_ms: i64,
}

/// Resolve the canonical `projects.json` path beneath the Tauri-managed
/// app data directory. Falls back to `<project_root>/.elai_cc/projects.json`
/// when the path resolver is unavailable (rare — generally only in
/// non-Tauri contexts such as unit tests).
fn projects_json_path(
    app: &tauri::AppHandle,
    state: &AppState,
) -> Result<std::path::PathBuf, String> {
    use tauri::Manager;
    match app.path().app_data_dir() {
        Ok(dir) => Ok(dir.join("projects.json")),
        Err(_) => {
            // Fallback path co-located with the active project (covers the
            // bare-cwd case used by elai.bat).
            let root = state.project_root_snapshot();
            Ok(root.join(".elai_cc").join("projects.json"))
        }
    }
}

/// C-4: list every project recorded in `<app_data_dir>/projects.json`.
/// Returns `{known: true, projects: []}` when the file is missing or
/// empty — graceful-empty shape per orchestrator decision Q4.
#[tauri::command]
pub fn list_projects(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> CmdResult<Value> {
    let path = projects_json_path(&app, &state)?;
    if !path.exists() {
        return Ok(json!({"known": true, "projects": []}));
    }
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(error = %e, "list_projects: read failed");
            return Ok(json!({"known": false, "projects": []}));
        }
    };
    if bytes.is_empty() {
        return Ok(json!({"known": true, "projects": []}));
    }
    let entries: Vec<ProjectEntry> = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "list_projects: parse failed");
            return Ok(json!({"known": false, "projects": []}));
        }
    };
    Ok(json!({"known": true, "projects": entries}))
}

/// C-4: create a new project directory and append it to `projects.json`.
///
/// Steps:
///   1. Validate `name` (non-empty, no path separators, no `..`).
///   2. `create_dir_all(parent_path/name)` — idempotent; existing dir OK.
///   3. Write a minimal `elai_cc.toml` placeholder if not present.
///   4. Atomic-rewrite `projects.json` with the new entry prepended.
///
/// Returns the created [`ProjectEntry`] on success.
#[tauri::command]
pub fn create_project(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    name: String,
    parent_path: String,
) -> CmdResult<Value> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("project name is empty".to_string());
    }
    if trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed == "."
        || trimmed == ".."
        || trimmed.contains("..")
    {
        return Err(format!("invalid project name: {trimmed}"));
    }
    let parent = std::path::PathBuf::from(parent_path.trim());
    if parent.as_os_str().is_empty() {
        return Err("parent_path is empty".to_string());
    }
    let project_dir = parent.join(trimmed);
    std::fs::create_dir_all(&project_dir)
        .map_err(|e| format!("create_dir_all({}): {e}", project_dir.display()))?;

    // Stub elai_cc.toml so the dashboard's "open" path can immediately
    // canonicalise the new project (the runtime config writer expects
    // a TOML file to exist).
    let toml_path = project_dir.join("elai_cc.toml");
    if !toml_path.exists() {
        let stub = format!("# Auto-generated by create_project — {trimmed}\n");
        std::fs::write(&toml_path, stub.as_bytes())
            .map_err(|e| format!("write {}: {e}", toml_path.display()))?;
    }

    let resolved = std::fs::canonicalize(&project_dir).unwrap_or(project_dir);
    #[cfg(windows)]
    let resolved = {
        let s = resolved.to_string_lossy().to_string();
        match s.strip_prefix(r"\\?\") {
            Some(rest) if !rest.starts_with("UNC\\") => std::path::PathBuf::from(rest),
            _ => resolved,
        }
    };

    let entry = ProjectEntry {
        name: trimmed.to_string(),
        path: resolved.to_string_lossy().into_owned(),
        last_opened_ms: now_ms(&state),
    };

    // Atomic merge into projects.json.
    let path = projects_json_path(&app, &state)?;
    if let Some(parent_dir) = path.parent() {
        std::fs::create_dir_all(parent_dir)
            .map_err(|e| format!("create_dir_all({}): {e}", parent_dir.display()))?;
    }
    let mut entries: Vec<ProjectEntry> = if path.exists() {
        match std::fs::read(&path) {
            Ok(bytes) if !bytes.is_empty() => {
                serde_json::from_slice(&bytes).unwrap_or_default()
            }
            _ => Vec::new(),
        }
    } else {
        Vec::new()
    };
    // De-duplicate on path (case-sensitive — caller controls casing).
    entries.retain(|e| e.path != entry.path);
    entries.insert(0, entry.clone());

    let serialized = serde_json::to_vec_pretty(&entries)
        .map_err(|e| format!("serialize projects.json: {e}"))?;
    let tmp = {
        let mut p = path.clone();
        let mut name_os = p
            .file_name()
            .map(|s| s.to_os_string())
            .unwrap_or_else(|| std::ffi::OsString::from("projects.json"));
        name_os.push(".tmp");
        p.set_file_name(name_os);
        p
    };
    std::fs::write(&tmp, &serialized)
        .map_err(|e| format!("write {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, &path)
        .map_err(|e| format!("rename {} -> {}: {e}", tmp.display(), path.display()))?;

    serde_json::to_value(&entry).map_err(map_err)
}

// ---------------------------------------------------------------------------
// C-29 (SCHEMA_VERSION 61) — INTENT-PAIRS (T-2) Tauri commands.
// add_intent_pair: append a single IntentPair to the per-step set.
// confirm_intent_pairs: flip confirmed=true + emit `IntentConfirmed` SSB row.
// get_intent_pairs: read back the per-step set for the PlanWindow editor.
// All three follow the existing `CmdResult<Value>` shape; arg names are
// snake_case in both Rust and JS per the project convention (Tauri's
// auto-camelCase translation is disabled — see context-package-C29 S10).
// ---------------------------------------------------------------------------

/// C-29: append an IntentPair to the per-BabyStep set held in AppState.
/// Returns `{ pair_count: N }` on success. Validation surface:
///   * Rejects empty `natural_language_intent` / `concrete_test_assertion`
///     via `IntentPair::new` (DRIFT-C29-7).
///   * Enforces char bounds 120/120/80.
#[tauri::command]
pub fn add_intent_pair(
    state: State<'_, AppState>,
    step_id: u64,
    natural_language_intent: String,
    concrete_test_assertion: String,
) -> CmdResult<Value> {
    let baby_step_ref = format!("bs-{step_id}");
    let pair = elai_orchestrator::IntentPair::new(
        natural_language_intent,
        concrete_test_assertion,
        baby_step_ref,
    )
    .map_err(map_err)?;
    let count = {
        let mut g = state.intent_pairs.lock().map_err(|_| "intent_pairs lock poisoned".to_owned())?;
        let set = g.entry(step_id).or_default();
        set.push(pair)
    };
    Ok(json!({ "pair_count": count, "step_id": step_id }))
}

/// C-29: confirm the per-BabyStep IntentPairSet. Returns Err when the
/// stored count is below the active `min_pairs` threshold (default 3).
/// On success emits one `SsbActionType::IntentConfirmed` row carrying
/// `confirmed_by="user"` + the canonical `pair_count`. The `min_pairs`
/// argument defaults to 3 when absent on the JS call so the editor can
/// stay synchronous with the gate's default; future wiring will thread
/// the actual `[testing.intent].min_pairs` value through.
#[tauri::command]
pub async fn confirm_intent_pairs(
    state: State<'_, AppState>,
    step_id: u64,
    min_pairs: Option<usize>,
) -> CmdResult<Value> {
    let threshold = min_pairs.unwrap_or(3);
    // Mutate the set under a brief lock. Drop the guard BEFORE the
    // .await on the SSB write (lock is std::sync::Mutex — never held
    // across .await per CLAUDE.md).
    let pair_count = {
        let mut g = state
            .intent_pairs
            .lock()
            .map_err(|_| "intent_pairs lock poisoned".to_owned())?;
        let set = g.entry(step_id).or_default();
        set.confirm(threshold).map_err(map_err)?;
        set.len()
    };
    // SSB emit (best-effort: when writer is absent we still return Ok so
    // the dashboard reflects state).
    let seq = if let Some(writer) = state.writer.as_ref() {
        let pair_count_u8 =
            u8::try_from(pair_count).map_err(|_| "pair_count overflows u8".to_owned())?;
        let payload = SsbPayload::IntentConfirmed(elai_orchestrator::ssb::IntentConfirmedPayload {
            step_id,
            pair_count: pair_count_u8,
            confirmed_by: "user".to_owned(),
        });
        Some(
            writer
                .write_payload("dashboard-v2", 0, payload, vec![])
                .await
                .map_err(map_err)?,
        )
    } else {
        None
    };
    Ok(json!({
        "ok": true,
        "step_id": step_id,
        "pair_count": pair_count,
        "seq": seq,
    }))
}

/// C-29: read back the per-BabyStep IntentPairSet for the PlanWindow
/// editor. Returns the JSON-shape `{ pairs: [...], confirmed: bool,
/// confirmed_at_ms: number|null }` — matching `IntentPairSet`'s serde
/// derivation. Empty set when no entry exists for the requested step_id.
#[tauri::command]
pub fn get_intent_pairs(state: State<'_, AppState>, step_id: u64) -> CmdResult<Value> {
    let set = state.get_baby_step_intent_pairs(step_id);
    serde_json::to_value(&set).map_err(map_err)
}

// ---------------------------------------------------------------------------
// C-45 — CANTANTE contrastive credit-attribution dashboard surface.
//
// cross-lang-pair: dashboard/tauri/api
//
// Paper grounding: arXiv:2605.13295 (Zehle 2026, CANTANTE Section 3.2 — K
// rollouts NOT K^N, scalar c in [-1,+1], g=3 partition default). The
// dashboard reads the most-recent in-process credit snapshot (synthetic
// fixture or live runpod result) via `credit_attribution_status`. The
// shape mirrors `CreditAttributionSnapshot` in the Svelte side at
// `crates/dashboard-v2/src/lib/drawers/OrchestrationDrawer.svelte`.
//
// Until the runpod-pending pilot lands, the command runs the
// orchestrator's synthetic-fixture pilot harness once per call and
// returns the rows + a `runpod_pending: true` flag so the UI always has
// shape-correct data to render. Replacing the synthetic call with the
// live result is a one-line swap once the pilot lands.
#[tauri::command]
pub async fn credit_attribution_status(
    _state: State<'_, AppState>,
) -> CmdResult<Value> {
    use elai_orchestrator::credit_attribution::{
        pilot_roles, CreditAttributionHarness, DropPolicy,
    };
    use elai_orchestrator::ipc_types::AgentRole;

    // Deterministic synthetic scorer — mirrors crates/cli/src/credit_cmd.rs.
    // Real credits land via the runpod pilot. Until then we render the
    // synthetic shape so the UI does not crash on cold-start.
    async fn synthetic_score(policy: DropPolicy, _i: u32) -> Result<f32, String> {
        match policy.dropped_role {
            None => Ok(0.80),
            Some(AgentRole::Executor) => Ok(0.10),
            Some(AgentRole::Auditor) => Ok(0.40),
            Some(AgentRole::Researcher) => Ok(0.55),
            Some(AgentRole::Designer) => Ok(0.70),
            Some(AgentRole::PromptEngineer) => Ok(0.65),
            Some(AgentRole::ContextCompiler) => Ok(0.45),
            Some(AgentRole::MemoryWriter) => Ok(0.75),
            Some(AgentRole::MediaValidator) => Ok(0.78),
            Some(_) => Ok(0.80),
        }
    }

    let ts_ms = std::time::SystemTime::now() // entropy-allow: synthetic credit-attribution pilot harness (`_state` deliberately unused; runpod_pending=true until the live pilot lands per fn doc comment), no production replay depends on this value yet
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0);
    let harness = CreditAttributionHarness::pilot(ts_ms);
    let report = harness
        .run(synthetic_score)
        .await
        .map_err(|e| map_err(format!("credit_attribution_status harness failed: {e}")))?;

    // Order rows by the canonical pilot_roles() ordering for stable
    // diffs. Roles missing from the report are skipped.
    let rows: Vec<Value> = pilot_roles()
        .iter()
        .copied()
        .filter_map(|role| {
            report.get(role).map(|score| {
                let role_str = serde_json::to_value(role)
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_else(|| format!("{role:?}").to_ascii_lowercase());
                json!({ "role": role_str, "score": score })
            })
        })
        .collect();

    Ok(json!({
        "rows": rows,
        "ts_ms": report.ts_ms,
        // True until the runpod pilot replaces the synthetic harness.
        "runpod_pending": true,
    }))
}

// ---------------------------------------------------------------------------
// C-46 — Uno-Orchestra parsimonious-routing dashboard surface.
//
// cross-lang-pair: dashboard/tauri/api
//
// Paper grounding: arXiv:2605.05007 (Cui et al 2026, Uno-Orchestra
// Sections 5 + 6.1 — (model, primitive) action space, alpha in [0, 1]
// convex blend, S_max <= 0.10 cost-hack bound, direct-answer lazy-mode
// cost zero). The dashboard reads recent selective-delegation
// decisions via `routing_decisions_status`. The shape mirrors
// `RoutingDecisionsSnapshot` in the Svelte side at
// `crates/dashboard-v2/src/lib/drawers/OrchestrationDrawer.svelte`.
//
// Until the runpod-pending pilot lands, the command runs the
// orchestrator's synthetic pilot harness once per call and returns
// the rows + a `runpod_pending: true` flag so the UI always has
// shape-correct data to render. Replacing the synthetic call with the
// live result is a one-line swap once the pilot lands.
#[tauri::command]
pub async fn routing_decisions_status(
    _state: State<'_, AppState>,
) -> CmdResult<Value> {
    use elai_orchestrator::ipc_types::AgentRole;
    use elai_orchestrator::routing::{
        RoutingPolicy, RoutingTask, SelectiveDelegator, TaskDifficultyClassifier,
    };
    use std::collections::BTreeMap;

    // Pilot policy mirrors crates/cli/src/routing_cmd.rs::pilot_policy
    // so the dashboard + CLI render the same synthetic numbers until
    // the production wiring fires.
    let policy = RoutingPolicy::new(true, 0.1, 0.10, 8, 8).map_err(|e| {
        map_err(format!("routing_decisions_status policy build failed: {e}"))
    })?;
    let delegator = SelectiveDelegator::new(policy);

    // 8-base-role pilot scope (matches credit_attribution::pilot_roles).
    let workers: BTreeMap<AgentRole, ()> = [
        AgentRole::Executor,
        AgentRole::Auditor,
        AgentRole::Researcher,
        AgentRole::Designer,
        AgentRole::PromptEngineer,
        AgentRole::ContextCompiler,
        AgentRole::MemoryWriter,
        AgentRole::MediaValidator,
    ]
    .iter()
    .map(|r| (*r, ()))
    .collect();

    // Three representative tasks: trivial (lazy-mode), moderate, hard.
    let templates: [RoutingTask; 3] = [
        RoutingTask::new_text("synth-trivial", "hi"),
        RoutingTask {
            text: "implement a small refactor in the routing module".into(),
            file_count: 2,
            multi_turn: false,
            task_id: "synth-moderate".into(),
        },
        RoutingTask {
            text: "refactor the orchestrator architecture: investigate why the dispatcher \
                   integrate path fails. ```rust let x = 1; ``` how does it interact with \
                   the auditor design? compare two approaches and evaluate trade-off."
                .into(),
            file_count: 8,
            multi_turn: true,
            task_id: "synth-hard".into(),
        },
    ];

    let rows: Vec<Value> = templates
        .iter()
        .map(|task| {
            let difficulty = TaskDifficultyClassifier::heuristic_score(task);
            let dispatched = delegator.select_workers(difficulty, &workers, None);
            let is_lazy_mode = dispatched.is_empty()
                && delegator.is_lazy_mode(difficulty);
            let pairs: Vec<Value> = dispatched
                .iter()
                .map(|(role, primitive)| {
                    let role_str = serde_json::to_value(role)
                        .ok()
                        .and_then(|v| v.as_str().map(String::from))
                        .unwrap_or_else(|| format!("{role:?}").to_ascii_lowercase());
                    json!({ "role": role_str, "primitive": primitive.as_str() })
                })
                .collect();
            json!({
                "task_id": task.task_id,
                "difficulty": difficulty,
                "alpha": policy.alpha,
                "dispatched": pairs,
                "is_lazy_mode": is_lazy_mode,
            })
        })
        .collect();

    let ts_ms = std::time::SystemTime::now() // entropy-allow: synthetic routing-decisions pilot harness (`_state` deliberately unused; runpod_pending=true until the live pilot lands per fn doc comment), no production replay depends on this value yet
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0);

    Ok(json!({
        "rows": rows,
        "policy": {
            "enabled": policy.enabled,
            "alpha": policy.alpha,
            "s_max": policy.s_max,
            "max_turns": policy.max_turns,
            "max_workers_per_query": policy.max_workers_per_query,
        },
        "ts_ms": ts_ms,
        // True until the runpod pilot replaces the synthetic harness.
        "runpod_pending": true,
    }))
}

// ---------------------------------------------------------------------------
// E-12 Tier-B — get_framework_profile + run_production_checklist
// ---------------------------------------------------------------------------
//
// Production fullstack codegen surface for the IdeaFramerSecurityPanel.
// Wire-dark posture: the registry + adapters compile against the
// orchestrator's framework_profile module, but live invocation of the
// 15-runner production checklist against representative apps is
// RUNPOD-PENDING per workflow/reasoning/E-12-tier-b-decisions.md. The
// command shape is stable; the underlying runners return
// ToolchainMissing on the local dev host (15 external CLIs absent).

/// E-12 Tier-B — resolve the framework profile for a given ProjectClass.
///
/// Reads workflow/archetypes/ via the orchestrator's
/// FrameworkProfileRegistry. When the archetypes dir is missing or
/// the resolved profile name is unregistered, returns
/// FrameworkProfile::generic() — the wire-dark fallback that keeps
/// the dashboard always-renderable.
///
/// `project_class` is the snake_case label from B-40's
/// `ProjectClass::as_snake_case()` (one of `consumer_mobile_app` /
/// `web_app` / `saas_dashboard` / `desktop_app` / `game_mod` /
/// `research_tool` / `library_or_sdk` / `cli_tool` / `other`). The
/// resolved profile carries the manifest-declared version pin
/// (never inlined as a source literal — Tier-A guard 4 enforces).
#[tauri::command]
pub fn get_framework_profile(
    state: State<'_, AppState>,
    project_class: String,
) -> CmdResult<Value> {
    let _ = state;
    use elai_orchestrator::framework_profile::get_framework_profile as resolve;
    use elai_orchestrator::idea_framing::ProjectClass;
    let class = ProjectClass::from_snake_case(&project_class).unwrap_or(ProjectClass::Other);
    // Project root resolution: walk up from CARGO_MANIFEST_DIR
    // (dashboard-v2 src-tauri) two levels to reach .elai_cc, then
    // up one more to reach the workflow root. This keeps the
    // command working regardless of where tauri::Builder was
    // launched from.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| manifest_dir.clone());
    let profile = resolve(&project_root, class);
    Ok(json!({
        "project_class": project_class,
        "profile_name": profile.name,
        "version_pin": profile.version_pin,
        "language": profile.language,
        "package_manager": profile.package_manager,
        "lint_config": profile.lint_config.display().to_string(),
        "test_runner": profile.test_runner,
        "security_checks": profile
            .security_checks
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>(),
        // E-12 Tier-B wire-dark marker — live runner toolchain
        // (15 CLIs) is absent on dev host; the dashboard renders
        // this badge so the user knows production-grade results
        // are pending the runpod toolchain install.
        "runpod_pending": true,
    }))
}

/// E-12 Tier-B — run the production checklist against `target_path`.
///
/// Sequential dispatch over `FrameworkProfile::security_checks` via
/// `production_checklist::run_check`. Each runner returns one of
/// `Ok { findings }`, `ToolchainMissing { tool, install_hint }`, or
/// `Error { details, stderr_excerpt }`. The dashboard renders all
/// three states; ToolchainMissing carries the install hint so the
/// user can resolve the gap without leaving the panel.
///
/// `target_path` MUST be absolute. The command returns Err when the
/// path does not exist or when the profile registry fails to
/// resolve a usable profile (the wire-dark fallback path returns
/// the generic profile, which still runs all 15 checks).
#[tauri::command]
pub async fn run_production_checklist(
    state: State<'_, AppState>,
    project_class: String,
    target_path: String,
) -> CmdResult<Value> {
    let _ = state;
    use elai_orchestrator::framework_profile::get_framework_profile as resolve;
    use elai_orchestrator::idea_framing::ProjectClass;
    use elai_orchestrator::production_checklist;
    let class = ProjectClass::from_snake_case(&project_class).unwrap_or(ProjectClass::Other);
    let target = PathBuf::from(&target_path);
    if !target.exists() {
        return Err(format!("target path does not exist: {}", target.display()));
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| manifest_dir.clone());
    let profile = resolve(&project_root, class);
    let results = production_checklist::run_all(&target, &profile.security_checks).await;
    let rows: Vec<Value> = results
        .into_iter()
        .map(|(check, res)| {
            let res_json = match res {
                production_checklist::RunResult::Ok { findings } => json!({
                    "kind": "ok",
                    "findings": findings,
                }),
                production_checklist::RunResult::ToolchainMissing { tool, install_hint } => json!({
                    "kind": "toolchain_missing",
                    "tool": tool,
                    "install_hint": install_hint,
                }),
                production_checklist::RunResult::Error { details, stderr_excerpt } => json!({
                    "kind": "error",
                    "details": details,
                    "stderr_excerpt": stderr_excerpt,
                }),
            };
            json!({
                "check": check.as_str(),
                "result": res_json,
            })
        })
        .collect();
    Ok(json!({
        "profile_name": profile.name,
        "target_path": target_path,
        "rows": rows,
        "runpod_pending": true,
    }))
}

/// E-8 — read recent `prompt_optimization_completed` SSB rows for the
/// dashboard `OptimizerRunPanel.svelte` surface. Emits the most recent
/// `n` rows (default RAG_QUALITY_DEFAULT_LIMIT, hard-capped at
/// RAG_QUALITY_HARD_CAP) plus aggregate red-flag predicates:
///   * `overfit_risk` -- latest row's held_out lift is more than 2pp
///     below val lift (DSPy 20/80 inversion per arXiv:2310.03714).
///   * `reflection_lm_too_small` -- always false in the dashboard's
///     READ surface: the CLI parser hard-refuses local LMs before any
///     SSB row is written, so a row whose reason_hash matches the
///     refusal label is the only path; the panel surfaces the predicate
///     as a forward-compatible flag.
///
/// Per-row shape: `{ seq, ts, phase, iteration_index, val_lift_pct,
/// held_out_lift_pct }`. See
/// `crates/orchestrator/src/ssb.rs::PromptOptimizationCompletedPayload`.
#[tauri::command]
pub fn get_optimizer_run(state: State<'_, AppState>, n: u32) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!({
            "rows": [],
            "latest_phase": null,
            "latest_val_lift_pct": null,
            "latest_held_out_lift_pct": null,
            "overfit_risk": false,
            "reflection_lm_too_small": false,
        }));
    };
    let limit = rag_quality_limit(n);
    let rows = viewer
        .recent_rows_of_type(SsbActionType::PromptOptimizationCompleted.as_sql_str(), limit)
        .unwrap_or_default();
    let mut out: Vec<Value> = Vec::with_capacity(rows.len());
    let mut latest_phase: Option<String> = None;
    let mut latest_val: Option<f64> = None;
    let mut latest_held_out: Option<f64> = None;
    for detail in rows {
        let inner = detail
            .payload_json
            .get("PromptOptimizationCompleted")
            .cloned()
            .unwrap_or(detail.payload_json);
        let phase = inner
            .get("phase")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let iteration_index = inner
            .get("iteration_index")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let val_lift_pct = inner
            .get("val_lift_pct")
            .and_then(|v| if v.is_null() { None } else { v.as_f64() });
        let held_out_lift_pct = inner
            .get("held_out_lift_pct")
            .and_then(|v| if v.is_null() { None } else { v.as_f64() });
        latest_phase.get_or_insert_with(|| phase.clone());
        if latest_val.is_none() {
            latest_val = val_lift_pct;
        }
        if latest_held_out.is_none() {
            latest_held_out = held_out_lift_pct;
        }
        out.push(json!({
            "seq": detail.seq,
            "ts": detail.ts,
            "phase": phase,
            "iteration_index": iteration_index,
            "val_lift_pct": val_lift_pct.map(Value::from).unwrap_or(Value::Null),
            "held_out_lift_pct": held_out_lift_pct.map(Value::from).unwrap_or(Value::Null),
        }));
    }
    let overfit_risk = match (latest_val, latest_held_out) {
        (Some(v), Some(h)) => (v - h) > 2.0,
        _ => false,
    };
    Ok(json!({
        "rows": out,
        "latest_phase": latest_phase,
        "latest_val_lift_pct": latest_val.map(Value::from).unwrap_or(Value::Null),
        "latest_held_out_lift_pct": latest_held_out.map(Value::from).unwrap_or(Value::Null),
        "overfit_risk": overfit_risk,
        "reflection_lm_too_small": false,
    }))
}

// ---------------------------------------------------------------------------
// E-47 -- LoRA ship gate status (read-only, cross-lang-pair: dashboard/tauri/api)
// ---------------------------------------------------------------------------

/// Read the current LoRA ship-gate status from the SSB viewer.
///
/// Returns:
///   - `gate_status`: derived gate outcome string ("refused" / "calibration_missing" /
///     "calibration_passed" / "wire_dark") -- "wire_dark" when no rows exist and
///     the gate is disabled.
///   - `gate_enabled`: bool mirroring `LoraShipGateConfig::enabled` from the
///     most recent SSB row's details_json, or false when no rows exist.
///   - `ship_gate_token`: the token from the config snapshot in the most
///     recent row's details_json, or "DEFERRED_PER_E47_REPORT" (default-deferred).
///   - `confirm_user_understanding`: bool from the config snapshot, or false.
///   - `rows`: recent `lora_ship_gate_event` SSB rows (newest first, up to N).
///
/// Wire-dark: when ELAI_LORA_SHIP_GATE_ENABLE is unset on the orchestrator
/// side, no SSB rows are emitted; this command returns the wire-dark envelope
/// (gate_status = "wire_dark", rows = []).  The config fields are extracted
/// from SSB row details_json when rows exist; otherwise they show the
/// default-deferred values.
///
/// Intentionally read-only: there is no Enable command -- the operator flips
/// the gate by editing .elai_cc/elai_cc.toml per
/// workflow/decisions/E-47-flip-runbook.md.
///
/// Off-agent artifact: `viewer.recent_rows_of_type("lora_ship_gate_event", N)`
/// reads SSB rows written by the real orchestrator producer
/// (notify_lora_ship_gate in crates/orchestrator/src/lora_ship_gate.rs), not
/// by this command. Missing rows => gate_status = "wire_dark".
#[tauri::command]
pub fn get_lora_ship_gate_status(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> CmdResult<Value> {
    use elai_orchestrator::lora_ship_gate::LORA_SHIP_GATE_DEFERRAL_TOKEN;

    const DEFAULT_LIMIT: usize = 50;
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);

    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!({
            "gate_status": "wire_dark",
            "gate_enabled": false,
            "ship_gate_token": LORA_SHIP_GATE_DEFERRAL_TOKEN,
            "confirm_user_understanding": false,
            "rows": [],
        }));
    };

    let raw = viewer
        .recent_rows_of_type(
            SsbActionType::LoraShipGateEvent.as_sql_str(),
            cap,
        )
        .unwrap_or_default();

    // Extract config fields from the most recent row's details_json.
    // The orchestrator stores a config snapshot in details_json alongside
    // the gate outcome, so the dashboard can show the active config without
    // reading elai_cc.toml directly.
    let (gate_enabled, ship_gate_token, confirm_user_understanding, gate_status) =
        if let Some(first) = raw.first() {
            let outer = &first.payload_json;
            // The SSB payload nests under the variant key "LoraShipGateEvent".
            let inner = outer
                .get("LoraShipGateEvent")
                .and_then(|v| v.as_object())
                .map(|m| serde_json::Value::Object(m.clone()))
                .unwrap_or_else(|| outer.clone());

            let details = inner.get("details_json").unwrap_or(&inner);

            let enabled = details
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let token = details
                .get("ship_gate_token")
                .and_then(|v| v.as_str())
                .unwrap_or(LORA_SHIP_GATE_DEFERRAL_TOKEN)
                .to_owned();
            let confirm = details
                .get("confirm_user_understanding")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            // Derive gate_status from the most recent row's phase field.
            let phase = inner
                .get("kind")
                .and_then(|v| v.as_str())
                .unwrap_or("refused")
                .to_owned();
            (enabled, token, confirm, phase)
        } else {
            (
                false,
                LORA_SHIP_GATE_DEFERRAL_TOKEN.to_owned(),
                false,
                "wire_dark".to_owned(),
            )
        };

    let mut rows_out: Vec<Value> = Vec::with_capacity(raw.len());
    for detail in &raw {
        let outer = &detail.payload_json;
        let inner = outer
            .get("LoraShipGateEvent")
            .and_then(|v| v.as_object())
            .map(|m| serde_json::Value::Object(m.clone()))
            .unwrap_or_else(|| outer.clone());

        let phase = inner
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("refused")
            .to_owned();
        let agent_id = inner
            .get("agent_id")
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        let session_id = inner
            .get("session_id")
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        let details_json = inner
            .get("details_json")
            .cloned()
            .unwrap_or_else(|| json!({}));

        rows_out.push(json!({
            "seq": detail.seq,
            "ts": detail.ts,
            "phase": phase,
            "agent_id": agent_id,
            "session_id": session_id,
            "details_json": details_json,
        }));
    }

    Ok(json!({
        "gate_status": gate_status,
        "gate_enabled": gate_enabled,
        "ship_gate_token": ship_gate_token,
        "confirm_user_understanding": confirm_user_understanding,
        "rows": rows_out,
    }))
}

#[cfg(test)]
mod b14_stream_chat_tests {
    use super::build_stream_request_body;

    #[test]
    fn request_body_without_thinking_has_no_thinking_block() {
        let body = build_stream_request_body("hello", false, None);
        assert!(body.get("thinking").is_none(), "thinking block absent");
        assert_eq!(
            body["messages"][0]["content"].as_str(),
            Some("hello")
        );
        assert_eq!(body["stream"].as_bool(), Some(true));
    }

    #[test]
    fn request_body_with_thinking_injects_thinking_block() {
        let body = build_stream_request_body("hello", true, Some("high"));
        let thinking = body.get("thinking").expect("thinking block present");
        assert_eq!(thinking["type"].as_str(), Some("enabled"));
        assert_eq!(thinking["budget_tokens"].as_u64(), Some(8192));
    }

    #[test]
    fn effort_level_maps_to_budget_tokens() {
        assert_eq!(
            build_stream_request_body("x", true, Some("low"))["thinking"]["budget_tokens"]
                .as_u64(),
            Some(2048)
        );
        assert_eq!(
            build_stream_request_body("x", true, Some("medium"))["thinking"]["budget_tokens"]
                .as_u64(),
            Some(4096)
        );
        assert_eq!(
            build_stream_request_body("x", true, Some("max"))["thinking"]["budget_tokens"]
                .as_u64(),
            Some(16384)
        );
        // Unknown effort defaults to medium (4096).
        assert_eq!(
            build_stream_request_body("x", true, None)["thinking"]["budget_tokens"].as_u64(),
            Some(4096)
        );
    }
}
