//! `elai task` — non-interactive task submission for fork-exec callers
//! (SWE-Bench Pro v5 driver, agent harnesses, CI scripts).
//!
//! Thin shell over the production task_runner wiring already used by
//! `elai run`. The lifecycle is:
//!   1. Optionally apply per-role runtime overrides via the B-37 toml-edit
//!      path (`[agents.<role>.runtime] backend = "<canonical>"`).
//!   2. Build the same `RuntimeWiring` as `run_subcommand::run_async` but
//!      with a derived `task-<pid>` session id (vs the long-lived
//!      `cli-run-<pid>`).
//!   3. Subscribe to the SSB broadcast BEFORE starting the runner so no
//!      row is lost.
//!   4. Start `start_task_runner`, write one `TaskSubmitted` SSB row
//!      (same shape as the dashboard's `submit_task` Tauri command).
//!   5. Race the subscription loop against `max_wall_seconds`.
//!   6. Terminal signal: a `PhaseComplete` row for our project_id, OR the
//!      runner future itself returning. Timeout cancels the runner.
//!   7. Aggregate the collected rows into the v5 driver JSON contract
//!      and print one JSONL line to stdout.
//!
//! Reuses existing types. No new SSB action_type variants.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use elai_core::clock::{ClockProvider, OsClock};
use elai_core::project_id_from_path;
use elai_orchestrator::ssb::{decode_ssb_payload, TaskSubmittedPayload};
use elai_orchestrator::{
    default_ssb_path_for_base, start_task_runner, GateMode, Ssb, SsbActionType, SsbConfig,
    SsbEntry, SsbPayload,
};
use elai_providers::runtime::resolve_backend;
use serde::Serialize;
use thiserror::Error;
use tokio::sync::broadcast::error::RecvError;

use crate::runtime_builder::{
    apply_task_runner_wiring, build_task_runner_wiring, install_production_agent_pool,
};

/// Categories of `Finding(severity="error")` that signal `drive_task`
/// completed for the current `TaskSubmitted` without emitting `PhaseComplete`.
/// `start_task_runner` keeps its outer `rx.recv()` loop alive after every
/// task to wait for more `TaskSubmitted` rows, so without this allowlist
/// `elai task` would block to wall budget on any task the orchestrator
/// terminates via an error finding rather than a `PhaseComplete` row.
///
/// Sources:
///   `task_failed`            crates/orchestrator/src/task_runner.rs:4058
///                            (drive_task returned Err)
///   `grounding_sweep_block`  crates/orchestrator/src/task_runner.rs:5482
///                            (GroundingSweepGate blocked Planning>TestingFirst)
///
/// Transient `severity="error"` categories with retry paths
/// (`web_scaffold_failed`, `web_research_failed`) are deliberately omitted.
const TERMINAL_ERROR_FINDING_CATEGORIES: &[&str] =
    &["task_failed", "grounding_sweep_block"];

/// Wire-mode for `elai task --mode <m>`. Distinct from `GateMode` —
/// this is the SWE-Bench driver's external knob (local / hybrid /
/// cloud) and maps to `ELAI_GROUNDING_BACKEND` + the `--provider` axis
/// in `elai run`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskMode {
    Local,
    Hybrid,
    Cloud,
}

impl TaskMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskMode::Local => "local",
            TaskMode::Hybrid => "hybrid",
            TaskMode::Cloud => "cloud",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "local" => Ok(TaskMode::Local),
            "hybrid" => Ok(TaskMode::Hybrid),
            "cloud" => Ok(TaskMode::Cloud),
            other => Err(format!(
                "unsupported `--mode` value: {other} (expected local, hybrid, or cloud)"
            )),
        }
    }
}

/// Output format on stdout. The v5 driver always asks for `json`; `text`
/// is retained for operator debugging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskOutputFormat {
    Json,
    Text,
}

impl TaskOutputFormat {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "json" => Ok(TaskOutputFormat::Json),
            "text" => Ok(TaskOutputFormat::Text),
            other => Err(format!(
                "unsupported `--output-format` value: {other} (expected json or text)"
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskArgs {
    pub problem_statement: String,
    pub repo_root: PathBuf,
    pub mode: TaskMode,
    pub output_format: TaskOutputFormat,
    pub max_wall_seconds: u64,
    /// Optional per-role override for the executor role. Values are
    /// resolved via `elai_providers::runtime::resolve_backend` so the
    /// CLI surface matches `elai runtime set --role executor
    /// --backend <X>`.
    pub executor_runtime: Option<String>,
    /// Optional per-role override for the auditor role.
    pub auditor_runtime: Option<String>,
}

/// Errors produced by the `elai task` driver path. Mapped to a 4-arm
/// exit-code surface per the v5 driver contract:
///   0 = task completed (or refused; the JSON payload disambiguates)
///   1 = task failed (wiring/runtime/IO error)
///   2 = task timed out
///   3 = config error (bad mode, missing key, invalid override)
#[derive(Debug, Error)]
pub enum TaskCmdError {
    #[error("config error: {0}")]
    Config(String),
    #[error("task wiring failed: {0}")]
    Wiring(String),
    #[error("ssb IO failed: {0}")]
    SsbIo(String),
    #[error("task timed out after {0}s")]
    Timeout(u64),
    #[error("internal: {0}")]
    Internal(String),
}

impl TaskCmdError {
    pub fn exit_code(&self) -> i32 {
        match self {
            TaskCmdError::Config(_) => 3,
            TaskCmdError::Timeout(_) => 2,
            _ => 1,
        }
    }
}

// ---------------------------------------------------------------------------
// JSON summary contract (matches v5 SWE-Bench Pro driver spec)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct PhaseTransitionRecord {
    pub from: String,
    pub to: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentDispatchRecord {
    pub role: String,
    pub agent_id: String,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_class: Option<String>,
    pub seq: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RefusalRecord {
    pub category: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskSummary {
    pub task_id: String,
    pub outcome: String,
    pub patch: Option<String>,
    pub files_touched: Vec<String>,
    pub ssb_path: String,
    pub phase_transitions: Vec<PhaseTransitionRecord>,
    pub agent_dispatches: Vec<AgentDispatchRecord>,
    pub audit_verdict: Option<String>,
    pub refusal_decision: Option<RefusalRecord>,
    pub wall_ms: u64,
    pub cost_usd: f64,
    pub project_id: String,
    pub mode: String,
    pub findings_total: u32,
    pub findings_block: u32,
    pub phase_complete: bool,
    pub timed_out: bool,
}

// ---------------------------------------------------------------------------
// Per-role override persistence (B-37 toml-edit path, atomic via tmp+rename).
// ---------------------------------------------------------------------------

fn apply_role_override(
    project_root: &Path,
    role: &str,
    candidate: &str,
) -> Result<(), TaskCmdError> {
    let backend =
        resolve_backend(candidate).map_err(|e| TaskCmdError::Config(format!(
            "unknown backend `{}` for role `{}`: supported = {}",
            e.candidate,
            role,
            e.supported.join(", "),
        )))?;
    let canonical = backend.canonical_name();
    let config_path = project_root.join("elai_cc.toml");
    let original = if config_path.is_file() {
        std::fs::read_to_string(&config_path)
            .map_err(|e| TaskCmdError::Config(format!(
                "failed to read {}: {e}", config_path.display()
            )))?
    } else {
        String::new()
    };
    let mut doc: toml::Value = if original.is_empty() {
        toml::Value::Table(toml::value::Table::new())
    } else {
        toml::from_str(&original).map_err(|e| TaskCmdError::Config(format!(
            "failed to parse {}: {e}", config_path.display()
        )))?
    };
    set_role_runtime_in_toml(&mut doc, role, canonical);
    let rendered = toml::to_string_pretty(&doc)
        .map_err(|e| TaskCmdError::Config(format!("toml serialize failed: {e}")))?;
    atomic_write_tmp_rename(&config_path, rendered.as_bytes())?;
    tracing::info!(
        role = %role,
        backend = %canonical,
        path = %config_path.display(),
        "task: per-role runtime override persisted"
    );
    Ok(())
}

fn set_role_runtime_in_toml(doc: &mut toml::Value, role: &str, backend_name: &str) {
    let root = match doc.as_table_mut() {
        Some(t) => t,
        None => {
            *doc = toml::Value::Table(toml::value::Table::new());
            doc.as_table_mut().expect("just rewrote to table")
        }
    };
    let agents = root
        .entry("agents".to_string())
        .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
    let agents_t = agents.as_table_mut().expect("agents must be table");
    let role_entry = agents_t
        .entry(role.to_string())
        .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
    let role_t = role_entry.as_table_mut().expect("role must be table");
    let runtime_entry = role_t
        .entry("runtime".to_string())
        .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
    let runtime_t = runtime_entry.as_table_mut().expect("runtime must be table");
    runtime_t.insert(
        "backend".to_string(),
        toml::Value::String(backend_name.to_string()),
    );
}

fn atomic_write_tmp_rename(path: &Path, content: &[u8]) -> Result<(), TaskCmdError> {
    let tmp_path: PathBuf = {
        let mut p = path.to_path_buf();
        let new_name = match p.file_name() {
            Some(name) => {
                let mut s = name.to_os_string();
                s.push(".tmp");
                s
            }
            None => {
                return Err(TaskCmdError::Config(format!(
                    "config path has no filename: {}",
                    path.display()
                )));
            }
        };
        p.set_file_name(new_name);
        p
    };
    if let Some(parent) = tmp_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| TaskCmdError::Config(format!(
            "mkdir {}: {e}", parent.display()
        )))?;
    }
    std::fs::write(&tmp_path, content).map_err(|e| TaskCmdError::Config(format!(
        "temp write {}: {e}", tmp_path.display()
    )))?;
    std::fs::rename(&tmp_path, path).map_err(|e| TaskCmdError::Config(format!(
        "rename {} -> {}: {e}", tmp_path.display(), path.display()
    )))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Synchronous entry-point called from the CliAction::Task dispatcher.
// ---------------------------------------------------------------------------

pub(crate) fn run_task_subcommand(args: TaskArgs) -> Result<(), anyhow::Error> {
    // Build a dedicated multi-thread runtime mirroring run_subcommand. The
    // parent fn run() in main.rs is sync.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("elai-task")
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!("elai task: tokio runtime init failed: {e}"))?;

    match runtime.block_on(async move { run_task_async(args).await }) {
        Ok(()) => Ok(()),
        Err(err) => {
            let code = err.exit_code();
            tracing::error!(exit_code = code, error = %err, "elai task: exiting with error");
            // Map the structured error to a process exit code without
            // tripping the generic anyhow exit-1 path in main.rs.
            std::process::exit(code);
        }
    }
}

async fn run_task_async(args: TaskArgs) -> Result<(), TaskCmdError> {
    // B-15 Tranche E E.1 / DE1: build one local Arc<dyn ClockProvider> at
    // fn entry; 5 reads across this body (started_at + deadline +
    // remaining + tail_deadline + tail-loop probe). Inline OsClock would
    // also work since it is Copy zero-sized; an Arc<dyn ClockProvider>
    // matches the existing CLI precedent for multi-read flows and keeps
    // the symbol available for a future deterministic swap.
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let started_at = clock.instant_now();

    // 0. Resolve canonical paths + identifiers up-front.
    let repo_root = args.repo_root.clone();
    if !repo_root.is_dir() {
        return Err(TaskCmdError::Config(format!(
            "--repo-root must be an existing directory: {}",
            repo_root.display()
        )));
    }
    let project_id = project_id_from_path(&repo_root);
    let ssb_path = default_ssb_path_for_base(&repo_root);
    let task_id = derive_task_id(&repo_root);
    let session_id = format!("cli-task-{}", std::process::id());

    // 1. Apply per-role overrides (B-37 toml-edit). Caller-controlled axis.
    if let Some(backend) = args.executor_runtime.as_deref() {
        apply_role_override(&repo_root, "executor", backend)?;
    }
    if let Some(backend) = args.auditor_runtime.as_deref() {
        apply_role_override(&repo_root, "auditor", backend)?;
    }

    // 2. Map the wire mode to ELAI_GROUNDING_BACKEND so run_subcommand's
    //    cloud-vs-sidecar dispatch picks the right AnthropicClient.
    //    SAFETY: this runs before the multi-threaded runtime spawns any
    //    worker that reads the variable.
    let grounding_backend = match args.mode {
        TaskMode::Cloud => "cloud",
        TaskMode::Hybrid => "hybrid",
        TaskMode::Local => "local",
    };
    // SAFETY: env::set_var is unsafe on 2024 edition.
    unsafe {
        std::env::set_var("ELAI_GROUNDING_BACKEND", grounding_backend);
    }

    // 3. Cloud-mode key check (config-error gate).
    if matches!(args.mode, TaskMode::Cloud | TaskMode::Hybrid) {
        if std::env::var("ANTHROPIC_API_KEY").ok().filter(|v| !v.is_empty()).is_none() {
            return Err(TaskCmdError::Config(format!(
                "--mode {} requires ANTHROPIC_API_KEY env var to be set",
                args.mode.as_str()
            )));
        }
    }

    tracing::info!(
        project_id = %project_id,
        session_id = %session_id,
        repo_root = %repo_root.display(),
        ssb_path = %ssb_path.display(),
        task_id = %task_id,
        mode = %args.mode.as_str(),
        max_wall_seconds = args.max_wall_seconds,
        "elai task: starting"
    );

    // 4. Build production wiring (same shape as run_subcommand::run_async).
    let mut wiring = build_task_runner_wiring(&repo_root, &session_id)
        .await
        .map_err(|e| TaskCmdError::Wiring(format!("{e}")))?;

    // Provider axis: mirror --provider on `elai run`. The driver always
    // asks for hybrid when --mode=hybrid; local/cloud map straight across.
    let provider_axis = match args.mode {
        TaskMode::Cloud => "cloud",
        TaskMode::Hybrid => "hybrid",
        TaskMode::Local => "local",
    };
    wiring.provider_override = Some(provider_axis.to_string());
    wiring.gate_mode = Some(GateMode::Autonomous);

    let elai_config = crate::runtime_config::load_elai_config(&repo_root);
    let cloud_config = elai_config.cloud.to_cloud_config();
    wiring.cloud_config = Some(cloud_config);
    wiring.hook_dispatcher =
        Some(elai_orchestrator::runtime_setup::build_production_hook_dispatcher());

    // patch-1.14 — install the production agent pool so non-baby-step
    // dispatchers (per-level auditor, decomposer follow-ups, evidence-gate
    // sub-agents) have a real AgentRpcServer to talk to. without this the
    // orchestrator falls through to SyntheticBabyStepExecutor and any future
    // dispatch path that goes through the AgentRpcServer trait blocks on a
    // tokio channel with no sender. soft-fails to synthetic on missing agent
    // binary; matches `elai run` semantics exactly.
    install_production_agent_pool(&mut wiring, &repo_root, &session_id)
        .await
        .map_err(|e| TaskCmdError::Wiring(format!("agent pool install failed: {e}")))?;

    let ssb_writer: Arc<Ssb> = wiring.ssb.clone();

    // 5. Subscribe BEFORE starting the runner so no row is missed.
    let mut rx = ssb_writer.subscribe();

    let cursor_path = repo_root
        .join(elai_core::paths::ELAI_DATA_DIR_NAME)
        .join("sessions")
        .join(&session_id)
        .join("cursor.json");
    let cfg = apply_task_runner_wiring(cursor_path, &wiring);

    let (mut handle, cancel) = start_task_runner(ssb_writer.clone(), cfg);

    // 6. Submit the TaskSubmitted row matching dashboard's submit_task call shape.
    // B-15 Tranche E E.1: route through the local ClockProvider rather
    // than the free `chrono_like_now_ms` fn (which still exists for the
    // free-fn pattern but is now wired through OsClock).
    let submitted_at_ms = i64::try_from(clock.now_ms()).unwrap_or(i64::MAX);
    let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
        task: args.problem_statement.clone(),
        project_id: project_id.clone(),
        submitted_at_ms,
    });
    ssb_writer
        .write_payload(
            elai_orchestrator::task_runner::RUNNER_AGENT_ID,
            elai_orchestrator::task_runner::RUNNER_SUPERSTEP,
            payload,
            Vec::new(),
        )
        .await
        .map_err(|e| TaskCmdError::SsbIo(format!("TaskSubmitted write failed: {e}")))?;

    // 7. Race: collect rows until we see the terminal signal, hit the
    //    wall budget, or the runner exits on its own.
    let deadline = clock.instant_now() + Duration::from_secs(args.max_wall_seconds.max(1));
    let mut collected = CollectedRows::default();
    let mut timed_out = false;

    loop {
        let remaining = deadline.saturating_duration_since(clock.instant_now());
        if remaining.is_zero() {
            timed_out = true;
            break;
        }
        tokio::select! {
            biased;
            res = (&mut handle) => {
                match res {
                    Ok(Ok(())) => {
                        tracing::info!("elai task: task_runner exited cleanly");
                    }
                    Ok(Err(runner_err)) => {
                        tracing::warn!(error = %runner_err, "elai task: task_runner returned error");
                    }
                    Err(join_err) => {
                        tracing::warn!(error = %join_err, "elai task: task_runner join failed");
                    }
                }
                break;
            }
            recv = rx.recv() => match recv {
                Ok(entry) => {
                    if entry.project_id != project_id {
                        continue;
                    }
                    collected.absorb(entry);
                    // patch-1.14 — `start_task_runner` is a forever-loop on
                    // its outer `rx.recv()` so the runner future never
                    // returns between tasks. break on either a terminal
                    // `PhaseComplete` row (happy path) or a terminal error
                    // finding (`grounding_sweep_block`, `task_failed`); both
                    // mean `drive_task` is done for this `TaskSubmitted` and
                    // no further rows will arrive without a fresh submit.
                    if collected.phase_complete || collected.terminal_error.is_some() {
                        // Give the runner a brief tail to flush any
                        // trailing FileWrite/Finding rows after the
                        // terminal signal.
                        let tail_deadline = clock.instant_now() + Duration::from_millis(500);
                        while clock.instant_now() < tail_deadline {
                            match tokio::time::timeout(
                                Duration::from_millis(50),
                                rx.recv(),
                            ).await {
                                Ok(Ok(extra)) => {
                                    if extra.project_id == project_id {
                                        collected.absorb(extra);
                                    }
                                }
                                _ => break,
                            }
                        }
                        break;
                    }
                }
                Err(RecvError::Lagged(_)) => {
                    // Best-effort sink; on lag we continue tailing.
                    continue;
                }
                Err(RecvError::Closed) => break,
            },
            _ = tokio::time::sleep(remaining) => {
                timed_out = true;
                break;
            }
        }
    }

    // 8. Cancel runner if it is still alive.
    cancel.cancel();
    let _ = tokio::time::timeout(Duration::from_secs(5), &mut handle).await;

    if timed_out {
        // Still emit the summary so the v5 driver gets observable state.
        emit_summary(
            &args,
            &collected,
            &task_id,
            &project_id,
            &ssb_path,
            started_at,
            true,
        );
        return Err(TaskCmdError::Timeout(args.max_wall_seconds));
    }

    emit_summary(
        &args,
        &collected,
        &task_id,
        &project_id,
        &ssb_path,
        started_at,
        false,
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Row aggregation
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct CollectedRows {
    phase_transitions: Vec<PhaseTransitionRecord>,
    agent_dispatches: Vec<AgentDispatchRecord>,
    files_touched: Vec<String>,
    findings_total: u32,
    findings_block: u32,
    audit_verdict: Option<String>,
    refusal: Option<RefusalRecord>,
    cost_usd: f64,
    phase_complete: bool,
    /// patch-1.14 — set to the category string of the first terminal error
    /// finding observed (`task_failed`, `grounding_sweep_block`, ...). when
    /// `Some` the collect loop treats it as equivalent to `phase_complete`
    /// for the purpose of breaking out, but `emit_summary` reports
    /// `outcome="blocked"` rather than `outcome="completed"`.
    terminal_error: Option<String>,
}

impl CollectedRows {
    fn absorb(&mut self, entry: SsbEntry) {
        let payload = match decode_ssb_payload(&entry.payload) {
            Ok(p) => p,
            Err(_) => return,
        };
        match payload {
            SsbPayload::FsmTransition(p) => {
                self.phase_transitions.push(PhaseTransitionRecord {
                    from: p.from,
                    to: p.to,
                    reason: p.reason,
                });
            }
            SsbPayload::PhaseTransition(p) => {
                self.phase_transitions.push(PhaseTransitionRecord {
                    from: p.from_state,
                    to: p.to_state,
                    reason: p.evidence_summary,
                });
            }
            SsbPayload::AgentLifecycle(p) => {
                self.agent_dispatches.push(AgentDispatchRecord {
                    role: p.role,
                    agent_id: p.agent_id,
                    state: p.state,
                    death_class: p.death_class,
                    seq: entry.seq,
                    created_at: entry.created_at,
                });
            }
            SsbPayload::FileWrite(p) => {
                if !self.files_touched.iter().any(|f| f == &p.path) {
                    self.files_touched.push(p.path);
                }
            }
            SsbPayload::Finding(p) => {
                self.findings_total = self.findings_total.saturating_add(1);
                if p.severity.eq_ignore_ascii_case("BLOCK") {
                    self.findings_block = self.findings_block.saturating_add(1);
                }
                // patch-1.14 — terminal error detection. matches the allowlist
                // at the top of this file. first hit wins so the v5 driver
                // sees the same category string the orchestrator emitted.
                if self.terminal_error.is_none()
                    && p.severity.eq_ignore_ascii_case("error")
                    && TERMINAL_ERROR_FINDING_CATEGORIES
                        .iter()
                        .any(|cat| p.category.eq_ignore_ascii_case(cat))
                {
                    self.terminal_error = Some(p.category.clone());
                }
                if p.category.eq_ignore_ascii_case("refusal") && self.refusal.is_none() {
                    self.refusal = Some(RefusalRecord {
                        category: p.category,
                        message: p.message,
                        file_path: p.file_path,
                    });
                }
            }
            SsbPayload::PerLevelAuditVerdict(p) => {
                // Last verdict wins; the v5 driver only needs the
                // terminal value.
                self.audit_verdict = Some(p.verdict.to_ascii_uppercase());
            }
            SsbPayload::PhaseComplete(_) => {
                self.phase_complete = true;
            }
            SsbPayload::CloudSpend(p) => {
                self.cost_usd += p.cost_usd;
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Summary emission
// ---------------------------------------------------------------------------

fn emit_summary(
    args: &TaskArgs,
    collected: &CollectedRows,
    task_id: &str,
    project_id: &str,
    ssb_path: &Path,
    started_at: Instant,
    timed_out: bool,
) {
    let wall_ms = u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX);

    let outcome = if timed_out {
        "timeout"
    } else if collected.refusal.is_some() {
        "refused"
    } else if collected.terminal_error.is_some() || collected.findings_block > 0 {
        "blocked"
    } else if collected.phase_complete || !collected.files_touched.is_empty() {
        "completed"
    } else {
        "blocked"
    };

    let summary = TaskSummary {
        task_id: task_id.to_string(),
        outcome: outcome.to_string(),
        patch: None,
        files_touched: collected.files_touched.clone(),
        ssb_path: ssb_path.display().to_string(),
        phase_transitions: collected.phase_transitions.clone(),
        agent_dispatches: collected.agent_dispatches.clone(),
        audit_verdict: collected.audit_verdict.clone(),
        refusal_decision: collected.refusal.clone(),
        wall_ms,
        cost_usd: collected.cost_usd,
        project_id: project_id.to_string(),
        mode: args.mode.as_str().to_string(),
        findings_total: collected.findings_total,
        findings_block: collected.findings_block,
        phase_complete: collected.phase_complete,
        timed_out,
    };

    match args.output_format {
        TaskOutputFormat::Json => {
            match serde_json::to_string(&summary) {
                Ok(line) => println!("{line}"),
                Err(e) => {
                    tracing::error!(error = %e, "elai task: JSON serialize failed");
                    eprintln!("{{\"task_id\":\"{task_id}\",\"outcome\":\"blocked\",\"serialize_error\":\"{e}\"}}");
                }
            }
        }
        TaskOutputFormat::Text => {
            println!(
                "task_id={} outcome={} wall_ms={} files_touched={} cost_usd={:.4}",
                summary.task_id,
                summary.outcome,
                summary.wall_ms,
                summary.files_touched.len(),
                summary.cost_usd
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Derive a task id from the repo root. SWE-Bench Pro layout is
/// `/root/swe_eval_repos/<task-slug>/`; the slug is the leaf component.
fn derive_task_id(repo_root: &Path) -> String {
    repo_root
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "task".to_string())
}

// B-15 Tranche E E.1: migrated to ClockProvider via the zero-sized
// OsClock Copy impl. The trait's default `now_ms` absorbs the
// `.duration_since(UNIX_EPOCH).unwrap_or(0)` saturation
// (crates/core/src/clock.rs:59-64). Kept as a free fn for callers that
// want the chrono-like helper shape without threading a clock through.
#[allow(dead_code)]
fn chrono_like_now_ms() -> i64 {
    i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX)
}

// Quiet unused-import warnings for SsbActionType / SsbConfig — they are
// public types that this module references via decode_ssb_payload and
// the wiring; keeping them on the use line documents intent without
// triggering dead_code warnings.
#[allow(dead_code)]
fn _type_anchor(_: SsbActionType, _: SsbConfig) {}
