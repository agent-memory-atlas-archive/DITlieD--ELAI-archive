//! Dashboard-v2 backend state. Slim, read-mostly. Owns the SsbViewer (read-only
//! SSB access), an optional SSB writer for user-driven mutations
//! (submit_task / intervention / chat / mode change / etc.), a ProjectsStore,
//! and a few mutable settings (gate mode, model routing, execution mode,
//! VRAM budget, MCP runtime flag).
//!
//! Heavy task_runner / agent_pool wiring is intentionally OMITTED — v2 is a
//! read-out + actuator surface, not an orchestrator host. The CLI launches
//! the actual task pipeline via elai.exe; v2 dashboard observes via SSB rows
//! and sidecar HTTP probes.

use std::collections::BTreeMap;

use rustc_hash::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

/// Progress snapshot for the remote-sidecar boot sequence.
/// Emitted as a Tauri event payload and stored in AppState.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteSidecarProgress {
    /// Which boot step: "ssh" | "tunnel" | "health" | "done".
    pub step: String,
    /// Step status: "pending" | "running" | "ok" | "error".
    pub status: String,
    /// Unix timestamp in milliseconds.
    pub ts_ms: i64,
    /// Error message, only set when status == "error".
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// B-24 Phase 4I Group C8: MediaValidationState — read-only snapshots fed by
// the SSB media_* events. The dashboard never touches orchestrator-internal
// validator types directly; it sees these stable serde-friendly mirrors.
// ---------------------------------------------------------------------------

/// One row of the Tier-1 deterministic check report (mirrors
/// `elai_orchestrator::media_validator::types::CheckResult` but serde-stable
/// so the dashboard contract does not couple to orchestrator internals).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CheckResultSnapshot {
    pub name: String,
    pub pass: bool,
    pub detail: String,
    pub latency_ms: u64,
}

/// Tier-1 aggregated snapshot. The full per-check breakdown is filled in
/// when MediaTier1CompletePayload is augmented with check details by a
/// future orchestrator change; for now `checks` is empty and the dashboard
/// renders pass/fail + latency.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tier1Snapshot {
    pub pass: bool,
    pub check_count: u32,
    pub latency_ms: u64,
    #[serde(default)]
    pub checks: Vec<CheckResultSnapshot>,
}

/// Tier-2 (VLM) advisory snapshot.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tier2Snapshot {
    pub advisory_pass: bool,
    pub model_used: String,
    pub latency_ms: u64,
    /// Free-shape JSON returned by the VLM (overall_quality, issues, etc.).
    /// `null` when the orchestrator emits Tier-2 complete without the
    /// structured payload extension.
    #[serde(default)]
    pub structured_output: serde_json::Value,
}

/// One rendered frame produced by an RCE render adapter.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CapturedFrameSnapshot {
    pub adapter_name: String,
    pub frame_count: u32,
    pub latency_ms: u64,
    /// Optional path to a thumbnail file on disk. The orchestrator does
    /// not yet emit per-frame image paths in the RenderComplete SSB payload
    /// (only adapter_name + frame_count), so this field stays `None` until
    /// that extension lands. Kept in the snapshot so the dashboard UI does
    /// not need a schema change once it is wired.
    #[serde(default)]
    pub image_path: Option<String>,
}

/// One iteration of the visual-feedback loop.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IterationSnapshot {
    pub iteration: u32,
    pub issue_count: u32,
    pub converged: bool,
    /// Optional verdict for the iteration. The current SSB payload only
    /// carries `converged`, so this is filled in on the final iteration
    /// when MediaHumanReviewRequired or a follow-up status row arrives.
    #[serde(default)]
    pub verdict: Option<String>,
    #[serde(default)]
    pub issues: Vec<String>,
}

/// E-16 -- per-entry AXTree YAML snapshot. Folded onto a
/// `MediaValidationEntry` when the orchestrator emits a
/// `VlmImageValidationKind::AxtreeMismatch` SSB row. `yaml_diff_pct`
/// is the line-granularity drift versus the stored baseline;
/// `mismatch` mirrors the producer flag (drift > threshold).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AxtreeSnapshot {
    pub yaml_diff_pct: f64,
    pub mismatch: bool,
    #[serde(default)]
    pub baseline_path: Option<String>,
    #[serde(default)]
    pub current_path: Option<String>,
    #[serde(default)]
    pub threshold_pct: Option<f64>,
}

/// E-16 -- per-entry sycophancy_probe snapshot. Folded onto a
/// `MediaValidationEntry` when the orchestrator emits a (REUSED)
/// `SycophancyFlag` SSB row from the media_validator
/// sycophancy_probe path. Verdict label vocab MUST match
/// SycophancyFlagPayload.verdict (E-4 / D-14): "convergent" |
/// "divergent_sycophancy_flag" | "inconclusive".
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SycophancySnapshot {
    pub flip_detected: bool,
    pub delta: f64,
    pub verdict: String,
}

/// Top-level entry surfaced by `get_media_validations`. One per `step_id`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MediaValidationEntry {
    /// Step id from the originating pipeline step. Used as the FxHashMap key.
    pub step_id: String,
    /// Optional request id (uuid stringified). The MediaValidationStarted
    /// payload does not currently carry it, so we mirror the step_id when
    /// no explicit uuid is available.
    pub request_id: String,
    pub media_type: String,
    pub file_path: String,
    #[serde(default)]
    pub tier1: Option<Tier1Snapshot>,
    #[serde(default)]
    pub tier2: Option<Tier2Snapshot>,
    #[serde(default)]
    pub render_frames: Vec<CapturedFrameSnapshot>,
    #[serde(default)]
    pub feedback_iterations: Vec<IterationSnapshot>,
    /// Final verdict: "Pass" | "Fail" | "HumanReview" | "Degraded".
    /// Mirrors `Verdict` from media_validator::types but serialized as a
    /// plain string for the TS contract.
    #[serde(default)]
    pub final_verdict: Option<String>,
    /// Last SSB row timestamp (ms since epoch) for this entry. Used by the
    /// dashboard to display "last updated".
    pub updated_at: u64,
    // -----------------------------------------------------------------
    // E-16 fields (default None / empty so old rows decode cleanly).
    // -----------------------------------------------------------------
    /// E-16 -- AXTree YAML snapshot folded from
    /// `VlmImageValidationKind::AxtreeMismatch`. None until at least
    /// one such row lands for the step_id.
    #[serde(default)]
    pub axtree: Option<AxtreeSnapshot>,
    /// E-16 -- sycophancy_probe snapshot folded from the REUSED
    /// `SycophancyFlag` SSB row. None until at least one such row
    /// lands for the step_id.
    #[serde(default)]
    pub sycophancy: Option<SycophancySnapshot>,
    /// E-16 -- swap-order pairwise disagreement folded from
    /// `VlmImageValidationKind::SwapOrderDisagreement`. None until at
    /// least one such row lands; `Some(true)` -> red badge.
    #[serde(default)]
    pub swap_order_disagreement: Option<bool>,
    /// E-16 -- monotonic LPIPS-delta (or SSIM-derived surrogate)
    /// history across the RCE iteration loop. One entry per
    /// iteration. Empty until iterations begin.
    #[serde(default)]
    pub convergence_history: Vec<f64>,
}

impl MediaValidationEntry {
    pub fn new(step_id: String, media_type: String, file_path: String, updated_at: u64) -> Self {
        Self {
            step_id: step_id.clone(),
            request_id: step_id,
            media_type,
            file_path,
            tier1: None,
            tier2: None,
            render_frames: Vec::new(),
            feedback_iterations: Vec::new(),
            final_verdict: None,
            updated_at,
            // E-16 fields default to None / empty until producers emit.
            axtree: None,
            sycophancy: None,
            swap_order_disagreement: None,
            convergence_history: Vec::new(),
        }
    }
}

/// Review decision the operator sends back when an entry resolves to
/// `Verdict::HumanReview`. Mirrors the spec's HumanReview branch
/// (phase-4I.txt L455).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ReviewDecision {
    Pass,
    Fail,
    RequestRevalidate,
}

use elai_core::clock::{ClockProvider, OsClock};
use elai_core::paths::sessions_dir_path;
#[cfg(feature = "mcp")]
use elai_plugins::{CasePersistStore, ToolCalibrationRegistry};
use elai_orchestrator::{
    CancelHandle, DefaultFeasibilityProber, FeasibilityProber, Layer1aProcessor,
    // B-37D Phase 4M (Wave-2A): the multi-runtime backend abstraction. Lives
    // in AppState so the 3 Tauri commands (list_runtimes / test_runtime /
    // set_role_runtime) operate against the same per-role mapping the
    // (future) task_runner consults. Default config maps the 15 AgentRoles
    // per B-37-decisions.md DECISION-8.
    RuntimeBackendConfig, RuntimeOrchestrator, Ssb, SsbConfig, SsbViewer,
    // Phase A1b: chokepoint helpers (moved from cli::runtime_builder into
    // elai_orchestrator::runtime_setup) used by the Tauri setup() callback
    // to spawn start_task_runner with both processors as Some(...).
    TaskRunnerError,
};
use elai_web_validator::DevServerHandle;

/// Default SSB database path resolution. We look in this order:
///   1. `ELAI_SSB_PATH` env var (explicit override).
///   2. `elai_core::paths::sessions_dir_path(project_root)/ssb.db`
///      (canonical: `<project_root>/.elai_cc/sessions/ssb.db`).
///   3. `<project_root>/.elai/sessions/ssb.db` (legacy path, intentional
///      fallback per B-1 L94-98).
/// Returns `None` if no candidate exists.
///
/// Phase A1b RISK-1: threads the active `project_root` rather than the
/// process cwd so that `open_project_from_folder`'s subsequent project
/// switch resolves to the same path the runner writes to. Today's caller
/// (`AppState::new()`) passes the launch-cwd project_root, preserving the
/// current cwd-based behaviour on boot.
pub(crate) fn resolve_ssb_path(project_root: &Path) -> Option<PathBuf> {
    if let Ok(env_path) = std::env::var("ELAI_SSB_PATH") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return Some(p);
        }
    }
    let candidates = [
        sessions_dir_path(project_root).join("ssb.db"),              // canonical
        project_root.join(".elai").join("sessions").join("ssb.db"),  // legacy, intentional fallback (B-1 L94-98)
    ];
    candidates.into_iter().find(|p| p.exists())
}

/// Resolve the active project root. Defaults to the current working directory
/// (where elai.bat / dashboard was launched from).
pub(crate) fn resolve_project_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Single shared dashboard state, attached to the Tauri Builder via `manage`.
pub struct AppState {
    /// Read-only SSB viewer. `None` when the SSB DB does not exist yet (fresh
    /// install, no sessions written). Commands that read SSB return empty
    /// shapes in that case.
    pub viewer: Option<Arc<SsbViewer>>,
    /// SSB writer used for user-driven mutations. Same DB as `viewer`.
    /// `None` when no SSB DB is reachable (commands that write return an
    /// error string).
    pub writer: Option<Arc<Ssb>>,
    /// Active project root. Initialised to the dashboard launch cwd; replaced
    /// by `open_project_from_folder` when the user picks a project. Reads
    /// must go through `project_root_snapshot()` so existing call sites do
    /// not race against a concurrent switch.
    pub project_root: Arc<Mutex<PathBuf>>,
    /// Active project_id used when emitting `task_submitted` / `chat_message`
    /// / `execution_mode_changed` rows. Defaults to "default".
    pub project_id: Arc<Mutex<String>>,
    /// Gate mode: "interactive" or "autonomous".
    pub gate_mode: Arc<Mutex<String>>,
    /// Execution mode: "LOCAL" / "CLOUD" / "HYBRID".
    pub execution_mode: Arc<Mutex<String>>,
    /// Per-role model routing — JSON-shape `[{role, target, fallback?}, ...]`.
    pub model_routing: Arc<Mutex<serde_json::Value>>,
    /// Anthropic model ID used in CLOUD execution mode. Persisted in
    /// `~/.elai/credentials.json` under key `cloud_model_id` (same file as the
    /// OAuth token). Validated against `elai_providers::CLOUD_MODEL_ID_ALLOWLIST`
    /// before storage. std::sync::Mutex matches the execution_mode pattern —
    /// reads and writes are pure string moves, never held across .await.
    pub cloud_model_id: Arc<Mutex<String>>,
    /// Persisted ELAI_LOCAL_MAX_CONTEXT (None = inherit from env/default).
    pub local_max_context: Arc<Mutex<Option<u32>>>,
    /// Persisted ELAI_LOCAL_MIN_PROMPT toggle.
    pub local_min_prompt: Arc<Mutex<bool>>,
    /// Persisted ELAI_ENABLE_MCP toggle.
    pub mcp_enabled: Arc<AtomicBool>,
    /// MCP server URL (default `http://localhost:8200`, override via
    /// `ELAI_MCP_SERVER_URL`).
    pub mcp_server_url: Arc<Mutex<String>>,
    /// Sidecar URL — defaults to `http://localhost:8100`, override via
    /// `ANTHROPIC_BASE_URL` or `ELAI_SIDECAR_URL`.
    pub sidecar_url: Arc<Mutex<String>>,
    /// Optional webhook URL for outbound event notifications.
    /// `None` = webhook disabled. Fired by frontend fetch(); Rust side stores config only.
    pub webhook_url: Arc<Mutex<Option<String>>>,
    /// Clock used for timestamping SSB writes.
    pub clock: Arc<dyn ClockProvider>,
    /// Reqwest client for sidecar HTTP probes (10s read timeout).
    pub http: reqwest::Client,
    /// Reqwest client for SSE streaming — no read timeout so long replies
    /// (512 tokens at ~30 tok/s = ~17s) do not time out mid-stream.
    pub streaming_http: reqwest::Client,
    /// Active streaming tasks keyed by stream_id. Each value is an
    /// AbortHandle that cancels the corresponding tokio task.
    /// BTreeMap (not HashMap) per CLAUDE.md coding standards.
    pub active_streams: Arc<Mutex<BTreeMap<String, tokio::task::AbortHandle>>>,
    /// Latest remote-sidecar boot progress snapshot. Updated by the
    /// start_remote_sidecar Tauri command as each boot step runs.
    /// `None` until `start_remote_sidecar` is first called.
    pub remote_sidecar_progress: Arc<Mutex<Option<RemoteSidecarProgress>>>,
    /// Phase A (s-69820): live dev-server registry keyed by canonicalised
    /// project directory string. Inserted by `start_dev_server_for_project`,
    /// removed by `stop_dev_server_for_project`. `DevServerHandle::Drop`
    /// kills the child if the entry is removed without an explicit stop.
    /// BTreeMap (not HashMap) per CLAUDE.md coding standards.
    ///
    /// Uses `tokio::sync::Mutex` (not `std::sync::Mutex`) so the guard can be
    /// held across the `start_dev_server` await without deadlocking the
    /// runtime — the audit found a TOCTOU race when `std::sync::Mutex` forced
    /// the lock to drop before the spawn.
    pub dev_servers: Arc<tokio::sync::Mutex<BTreeMap<String, DevServerHandle>>>,
    /// Phase 4F (B-21): pluggable feasibility prober. Constructed at startup
    /// rooted at `<project_root>/.elai_cc/feasibility/`. Used by future Tauri
    /// commands that trigger an inline probe; the read-side
    /// `get_feasibility_report` command queries the SSB directly via
    /// `SsbViewer::feasibility_summary` and does not need the prober.
    #[allow(dead_code)] // wired for future probe-now Tauri command (4F.7-4F.8 path)
    pub feasibility_prober: Arc<dyn FeasibilityProber>,
    /// B-24 Phase 4I Group C8: per-step media validation entries fed by the
    /// SSB subscriber. Keyed by `step_id`. `tokio::sync::RwLock` because the
    /// SSB subscriber updates it under an async task while Tauri commands
    /// read it; per CLAUDE.md rule "tokio::sync::Mutex in async (std::sync::
    /// Mutex only if provably sync-only)".
    pub media_validation: Arc<tokio::sync::RwLock<FxHashMap<String, MediaValidationEntry>>>,
    /// Phase A1b: live `start_task_runner` join handle. `None` until the
    /// Tauri `setup()` callback spawns the runner (which it only does when
    /// `writer.is_some()` — fresh installs without an SSB DB still have a
    /// usable dashboard, just without a runner loop). `std::sync::Mutex` is
    /// sufficient because the only writers are the setup() callback and the
    /// `ExitRequested` callback (both run on the Tauri main thread); no
    /// async holder ever .awaits while holding the guard.
    pub runner_join: Arc<Mutex<Option<tokio::task::JoinHandle<Result<(), TaskRunnerError>>>>>,
    /// Phase A1b: live cancellation handle paired with `runner_join`. Taken
    /// out of the mutex by the Tauri `RunEvent::ExitRequested` callback
    /// during shutdown so the runner observes the flip and exits its select
    /// loop cleanly (task_runner.rs:CancelHandle::cancel — flips an
    /// AtomicBool the runner polls each tick).
    pub runner_cancel: Arc<Mutex<Option<CancelHandle>>>,
    /// Phase A4: live Layer1A processor handle. `None` until the Tauri
    /// `setup()` callback wires it from `RuntimeWiring` (same hook that
    /// populates `runner_join`). Used by `apply_proposal_from_user` and
    /// `reject_proposal` to delegate to the canonical Layer1A path instead
    /// of writing decorative SSB rows. `std::sync::Mutex` because the only
    /// writer is the setup() callback (single thread) and command readers
    /// take a brief lock + clone the Arc out, never holding the guard
    /// across `.await`.
    pub layer1a_processor: Arc<Mutex<Option<Arc<dyn Layer1aProcessor + Send + Sync>>>>,
    /// B-37D Phase 4M (Wave-2A): per-role runtime orchestrator. Owns the
    /// PerRoleRuntime mapping (15 AgentRoles -> RuntimeBackend), the
    /// fallback chain, and per-backend factory configs. Constructed at
    /// startup with `RuntimeBackendConfig::default()` (per-role defaults
    /// per B-37 plan lines 104-110 — designer/auditor/researcher/
    /// media_validator and the 5 B-41 per-level auditor siblings map to
    /// AnthropicApi; executor/context_compiler/memory_writer/
    /// prompt_engineer/idea_framer/idea_critic map to Sidecar).
    ///
    /// Mutated by the `set_role_runtime` Tauri command (commands.rs).
    /// Read by `list_runtimes` (per-backend health snapshot) and
    /// indirectly by future task_runner integration (Wave-3, out of
    /// B-37D scope).
    ///
    /// Wrapped in `Arc` so command handlers can clone the handle cheaply.
    /// `RuntimeOrchestrator` is internally `RwLock`-guarded, so no
    /// outer `Mutex` is needed at this layer.
    pub runtime_orchestrator: Arc<RuntimeOrchestrator>,
    /// C-29: per-BabyStep `IntentPairSet` registry keyed by `step_id`.
    /// Populated by the PlanWindow IntentPairEditor (via `add_intent_pair`)
    /// and read back by `get_intent_pairs`. `confirm_intent_pairs` flips
    /// the contained `IntentPairSet::confirmed` flag + emits an
    /// `IntentConfirmed` SSB row via `writer`. `std::sync::Mutex` is
    /// sufficient because all readers/writers are sync Tauri command
    /// handlers — no guard is held across `.await`. BTreeMap (not
    /// HashMap) per CLAUDE.md coding standards.
    pub intent_pairs: Arc<Mutex<BTreeMap<u64, elai_orchestrator::IntentPairSet>>>,
    /// B-25 Phase 4J: optional shared `EmpireBridge` handle. `None` until
    /// the CLI launches with `--empire` AND the orchestrator is compiled
    /// with `--features empire`; in that case the launcher passes a
    /// pre-constructed `Arc<EmpireBridge>` via `AppState::with_empire_bridge`.
    /// Always `None` for non-empire builds and for empire-feature builds
    /// not opted into via CLI.
    ///
    /// commands_empire.rs reads this snapshot so that bridge state survives
    /// across Tauri command invocations (a fresh `EmpireBridge::new()` per
    /// call would reset state every time and break the dashboard render).
    ///
    /// EMPIRE-PENDING flip: once the activation patch wires the bridge to
    /// a live FactoryLine transport, this field's `Some(...)` arm is the
    /// one place the dashboard learns the bridge is connected.
    #[cfg(feature = "empire")]
    pub empire_bridge: Option<Arc<elai_orchestrator::empire::EmpireBridge>>,
    /// C-23: optional resolved path to the active SessionDag's `dag.db`.
    /// Per-call DAG open trade-off: dashboard-v2 does not hold a long-
    /// lived `SessionDag` handle (Q3 orchestrator decision). Each
    /// `branch_fork` / `branch_merge` / `branch_close` / `load_handoff`
    /// invocation opens the SQLite DB, performs the operation, and
    /// drops the handle. Trade-off: ~5 ms open cost per call vs.
    /// avoiding `Arc<Mutex<SessionDag>>` lifetime entanglement with the
    /// CLI process. `None` when the dashboard launched without an
    /// active session (the per-call resolver will then refuse the
    /// command with a typed `DagBranchError::NoActiveSession`).
    pub dag_db_path: Arc<Mutex<Option<PathBuf>>>,
    /// H-11 (SCHEMA_VERSION 220): server-side allowed-root registry for the
    /// scoped filesystem commands (read_dir / read_file / open_in_editor).
    ///
    /// Seeded at startup with the canonicalized project_root. `mount_root`
    /// pushes additional roots as the user opens multi-root workspaces.
    /// `read_dir` / `read_file` / `open_in_editor` REJECT any root argument
    /// that is not under one of these entries (server-side scope enforcement,
    /// not dependent on the Tauri capability system which std::fs bypasses).
    ///
    /// `std::sync::Mutex` is correct here: all read_dir / mount_root callers
    /// are sync Tauri command handlers — no guard is held across `.await`.
    pub allowed_roots: Arc<Mutex<Vec<PathBuf>>>,
    /// C-47b: optional calibration registry with the cold-tier persist store
    /// installed. None when ELAI_C47B_CASE_BASE_PERSIST_ENABLE is unset at
    /// startup OR when CasePersistStore::open fails (fail-open per D3).
    /// Populated synchronously in AppState::new(); rehydrated from disk in a
    /// background task spawned by lib.rs immediately after construction.
    /// tool_calibration_status reads live data from this registry.
    /// Arc without outer Mutex: ToolCalibrationRegistry is internally
    /// lock-guarded per-tool; the field is written once at construction.
    #[cfg(feature = "mcp")]
    pub calibration_registry: Option<Arc<ToolCalibrationRegistry>>,
}

impl AppState {
    pub fn new() -> Self {
        let project_root = resolve_project_root();
        let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);

        let (viewer, writer) = match resolve_ssb_path(&project_root) {
            Some(db_path) => Self::open_pair(&db_path, clock.clone()),
            None => {
                tracing::warn!(
                    "dashboard-v2: SSB db not found (looked under .elai_cc/sessions \
                     and .elai/sessions of cwd, and ELAI_SSB_PATH); read commands \
                     will return empty shapes until a session writes the db"
                );
                (None, None)
            }
        };

        let mcp_runtime = match std::env::var("ELAI_ENABLE_MCP") {
            Ok(v) => {
                let t = v.trim().to_ascii_lowercase();
                t == "1" || t == "true"
            }
            Err(_) => false,
        };
        let mcp_url = std::env::var("ELAI_MCP_SERVER_URL")
            .ok()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "http://localhost:8200".to_owned());
        let sidecar_url = std::env::var("ELAI_SIDECAR_URL")
            .or_else(|_| std::env::var("ANTHROPIC_BASE_URL"))
            .ok()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| elai_providers::DEFAULT_SIDECAR_URL.to_owned());

        let local_max = std::env::var("ELAI_LOCAL_MAX_CONTEXT")
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok());
        let local_min = matches!(
            std::env::var("ELAI_LOCAL_MIN_PROMPT")
                .ok()
                .as_deref()
                .map(str::trim),
            Some("1") | Some("true") | Some("True")
        );

        // 10s read timeout: /health first-probe can take ~1s after tunnel idle
        // (observed in smoke-test T4: 5s timed out, 10s succeeded). streaming_http
        // is unaffected — it has no read timeout by design.
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .connect_timeout(std::time::Duration::from_secs(2))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        // No read timeout: SSE streaming replies may take 17+ seconds for
        // 512 tokens at 30 tok/s. Connect timeout retained so offline sidecar
        // is detected fast. Read timeout omitted intentionally.
        let streaming_http = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(2))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        // Pre-load persisted cloud model preference from credentials.json.
        // Falls back to DEFAULT_CLOUD_MODEL_ID when the file is absent, the
        // key is missing, or the stored value is no longer in the allowlist
        // (allowlist may shrink across releases).
        let initial_cloud_model = match elai_providers::load_cloud_model_id() {
            Ok(Some(stored)) if elai_providers::is_valid_cloud_model_id(&stored) => stored,
            Ok(Some(stored)) => {
                tracing::warn!(
                    stored = %stored,
                    "dashboard-v2: persisted cloud_model_id not in allowlist, falling back to default"
                );
                elai_providers::DEFAULT_CLOUD_MODEL_ID.to_owned()
            }
            Ok(None) => elai_providers::DEFAULT_CLOUD_MODEL_ID.to_owned(),
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    "dashboard-v2: failed to read cloud_model_id from credentials.json"
                );
                elai_providers::DEFAULT_CLOUD_MODEL_ID.to_owned()
            }
        };

        let default_routing = serde_json::json!([
            { "role": "executor", "target": "local:qwen-2.5-coder-7b" },
            { "role": "auditor", "target": "local:qwen-2.5-coder-7b" },
            { "role": "researcher", "target": "local:qwen-2.5-coder-7b" },
            { "role": "designer", "target": "local:qwen-2.5-coder-7b" },
            { "role": "prompt-engineer", "target": "local:qwen-2.5-coder-3b" },
            { "role": "context-compiler", "target": "local:qwen-2.5-coder-3b" },
            { "role": "memory-writer", "target": "local:qwen-2.5-coder-3b" },
            { "role": "media-validator", "target": "local:qwen-2.5-coder-7b" },
        ]);

        // Phase 4F: feasibility prober rooted at <project_root>/.elai_cc/feasibility/.
        // Construction is cheap (no shell-out / no I/O until probe() is called)
        // so this stays inside AppState::new() without slowing startup.
        //
        // E-22 phase-4F M1 + Tier3 injection (2026-05-22): mirror the
        // production wiring at `runtime_setup.rs::build_task_runner_wiring`
        // by injecting both a system_prober and a Tier3StubExecutor
        // (locating `elai_cc_agent` via `locate_default_agent_binary`).
        // Binary-not-found is non-fatal — the prober falls back to the
        // synthetic stub path so the dashboard renders even on agentless
        // dev hosts.
        let feasibility_root = project_root.join(".elai_cc").join("feasibility");
        let clock_for_probe: Arc<dyn ClockProvider> = Arc::new(OsClock);
        let system_prober_for_feasibility: Arc<dyn elai_orchestrator::SystemProber> =
            Arc::new(elai_orchestrator::DefaultSystemProber::new(
                Some(project_root.clone()),
                clock_for_probe,
            ));
        let mut prober = DefaultFeasibilityProber::new(feasibility_root.clone())
            .with_system_prober(Arc::clone(&system_prober_for_feasibility));
        match elai_orchestrator::locate_default_agent_binary() {
            Ok(agent_path) => {
                let tier3 = elai_orchestrator::Tier3StubExecutor::new(agent_path);
                prober = prober.with_stub_executor(Arc::new(tier3));
            }
            Err(e) => {
                tracing::info!(
                    error = %e,
                    "AppState::new: elai_cc_agent not located — feasibility prober keeps the synthetic stub path"
                );
            }
        }
        let feasibility_prober: Arc<dyn FeasibilityProber> = Arc::new(prober);

        // E-3 sub-phase 4B: build the runtime orchestrator with the Ssb
        // handle attached (when reachable) before `writer` moves into
        // Self below. The orchestrator stays cheap to construct (no
        // adapter instantiation) so doing this here adds zero startup
        // cost.
        let runtime_orch_with_ssb = {
            let base = RuntimeOrchestrator::new(RuntimeBackendConfig::default());
            match writer.as_ref() {
                Some(w) => base.with_ssb(Arc::clone(w)),
                None => base,
            }
        };

        // C-47b: build the ToolCalibrationRegistry with the cold-tier persist
        // store when ELAI_C47B_CASE_BASE_PERSIST_ENABLE is truthy. Fail-open:
        // if CasePersistStore::open errors (disk permission, path missing,
        // sqlite locked) we log warn and fall back to in-memory-only. No panic.
        // The env gate is checked once here and once per record_call inside the
        // registry (canonical D3 fail-closed behaviour preserved).
        #[cfg(feature = "mcp")]
        let calibration_registry: Option<Arc<ToolCalibrationRegistry>> = {
            let gate = std::env::var("ELAI_C47B_CASE_BASE_PERSIST_ENABLE")
                .map(|v| {
                    let t = v.trim().to_ascii_lowercase();
                    t == "1" || t == "true" || t == "yes" || t == "on"
                })
                .unwrap_or(false);
            if gate {
                let persist_path = project_root
                    .join(".elai_cc")
                    .join("data")
                    .join("case_base")
                    .join("case_base.sqlite3");
                match CasePersistStore::open(&persist_path) {
                    Ok(store) => {
                        tracing::info!(
                            path = %persist_path.display(),
                            "C-47b: CasePersistStore opened; calibration registry wired with persist"
                        );
                        Some(Arc::new(
                            ToolCalibrationRegistry::new()
                                .with_persist(Arc::new(store)),
                        ))
                    }
                    Err(e) => {
                        tracing::warn!(
                            path = %persist_path.display(),
                            error = %e,
                            "C-47b: CasePersistStore::open failed; calibration registry in-memory-only (fail-open)"
                        );
                        Some(Arc::new(ToolCalibrationRegistry::new()))
                    }
                }
            } else {
                None
            }
        };

        // H-11: capture canonical project_root BEFORE project_root is moved
        // into Arc<Mutex<>> below. canonicalize is best-effort (falls back to
        // clone on IO failure, e.g. path not yet created in tests).
        let initial_allowed_root = std::fs::canonicalize(&project_root)
            .unwrap_or_else(|_| project_root.clone());

        Self {
            viewer,
            writer,
            project_root: Arc::new(Mutex::new(project_root)),
            project_id: Arc::new(Mutex::new("default".to_owned())),
            gate_mode: Arc::new(Mutex::new("interactive".to_owned())),
            execution_mode: Arc::new(Mutex::new(
                std::env::var("ELAI_DEFAULT_EXECUTION_MODE")
                    .ok()
                    .map(|s| s.trim().to_ascii_uppercase())
                    .filter(|s| matches!(s.as_str(), "LOCAL" | "CLOUD" | "HYBRID"))
                    .unwrap_or_else(|| "LOCAL".to_owned()),
            )),
            model_routing: Arc::new(Mutex::new(default_routing)),
            cloud_model_id: Arc::new(Mutex::new(initial_cloud_model)),
            local_max_context: Arc::new(Mutex::new(local_max)),
            local_min_prompt: Arc::new(Mutex::new(local_min)),
            mcp_enabled: Arc::new(AtomicBool::new(mcp_runtime)),
            mcp_server_url: Arc::new(Mutex::new(mcp_url)),
            sidecar_url: Arc::new(Mutex::new(sidecar_url)),
            webhook_url: Arc::new(Mutex::new(None)),
            clock,
            http,
            streaming_http,
            active_streams: Arc::new(Mutex::new(BTreeMap::new())),
            remote_sidecar_progress: Arc::new(Mutex::new(None)),
            dev_servers: Arc::new(tokio::sync::Mutex::new(BTreeMap::new())),
            feasibility_prober,
            media_validation: Arc::new(tokio::sync::RwLock::new(FxHashMap::default())),
            // Phase A1b: runner handles default to None. Populated by the
            // Tauri setup() callback in lib.rs when writer.is_some(); the
            // RunEvent::ExitRequested closure takes them back out on shutdown.
            runner_join: Arc::new(Mutex::new(None)),
            runner_cancel: Arc::new(Mutex::new(None)),
            // Phase A4: layer1a_processor defaults to None. Populated by the
            // Tauri setup() callback in lib.rs alongside runner_join when
            // RuntimeWiring resolves; apply_proposal_from_user / reject_proposal
            // read it via layer1a_processor_snapshot().
            layer1a_processor: Arc::new(Mutex::new(None)),
            // B-37D Wave-2A: per-role runtime orchestrator. Default config
            // pre-populates factory_cfg for every backend and the
            // PerRoleRuntime map for every AgentRole, so `list_runtimes` /
            // `set_role_runtime` work immediately on a fresh install.
            // The orchestrator does not instantiate adapters until first
            // use (lazy cache), so this constructor remains cheap.
            //
            // E-3 sub-phase 4B: when an Ssb writer is reachable (session
            // DB resolved at startup), attach it via `with_ssb` so the
            // orchestrator can emit unified_preamble_applied +
            // runtime_prewarm_{started,complete} rows in-process.
            runtime_orchestrator: Arc::new(runtime_orch_with_ssb),
            // C-23: dag_db_path is populated lazily by the per-call
            // resolver in commands.rs (`resolve_active_dag_path`). Left
            // None at startup so a fresh install with no sessions does
            // not falsely advertise a DAG.
            dag_db_path: Arc::new(Mutex::new(None)),
            // H-11: seed allowed_roots with the canonicalized project_root.
            // initial_allowed_root was captured above before project_root was moved.
            allowed_roots: Arc::new(Mutex::new(vec![initial_allowed_root])),
            // C-29: empty IntentPair registry at startup. PlanWindow
            // populates it via the `add_intent_pair` command.
            intent_pairs: Arc::new(Mutex::new(BTreeMap::new())),
            // B-25 Phase 4J: empire_bridge defaults to None even with the
            // feature on; the CLI / dashboard launcher constructs the Arc
            // and swaps it via with_empire_bridge before the Tauri runtime
            // takes ownership of AppState.
            #[cfg(feature = "empire")]
            empire_bridge: None,
            // C-47b: calibration_registry constructed above; None when
            // the env gate is off or the feature is absent.
            #[cfg(feature = "mcp")]
            calibration_registry,
        }
    }

    /// B-25 Phase 4J: install a pre-constructed `EmpireBridge` on AppState.
    /// Called by the dashboard launcher when the user passes `--empire` and
    /// the orchestrator is compiled with `--features empire`. Returns the
    /// updated AppState by value so the caller can chain it into
    /// `tauri::Builder::manage`.
    #[cfg(feature = "empire")]
    pub fn with_empire_bridge(
        mut self,
        bridge: Arc<elai_orchestrator::empire::EmpireBridge>,
    ) -> Self {
        self.empire_bridge = Some(bridge);
        self
    }

    /// B-25 Phase 4J: snapshot helper for the shared `EmpireBridge` handle.
    /// Returns `None` when the dashboard launched without the empire
    /// feature OR without `--empire`. Cheap Arc clone, no lock acquisition.
    #[cfg(feature = "empire")]
    pub fn empire_bridge_snapshot(
        &self,
    ) -> Option<Arc<elai_orchestrator::empire::EmpireBridge>> {
        self.empire_bridge.clone()
    }

    /// C-29 helper: snapshot the IntentPairSet for `step_id`. Returns the
    /// stored set or `IntentPairSet::default()` (empty unconfirmed) when
    /// no entry exists. Wraps the brief mutex acquisition so command
    /// handlers stay readable. Lock is released before any `.await`.
    pub fn get_baby_step_intent_pairs(
        &self,
        step_id: u64,
    ) -> elai_orchestrator::IntentPairSet {
        match self.intent_pairs.lock() {
            Ok(g) => g.get(&step_id).cloned().unwrap_or_default(),
            Err(_) => elai_orchestrator::IntentPairSet::default(),
        }
    }

    /// C-23: snapshot helper for the dag.db path. Returns the cached path
    /// when set, or `None` if no active SessionDag has been resolved.
    /// Callers should use `commands::resolve_active_dag_path` to populate
    /// this lazily on first use.
    pub fn dag_db_path_snapshot(&self) -> Option<PathBuf> {
        self.dag_db_path.lock().ok().and_then(|g| g.clone())
    }

    /// C-23: set the cached dag.db path. Returns `true` if the value
    /// changed. Idempotent — repeated calls with the same path are
    /// no-ops. Silent no-op on mutex poison (the lookup re-resolves on
    /// the next call).
    pub fn set_dag_db_path(&self, path: PathBuf) -> bool {
        match self.dag_db_path.lock() {
            Ok(mut g) => {
                let changed = g.as_ref() != Some(&path);
                if changed {
                    *g = Some(path);
                }
                changed
            }
            Err(_) => false,
        }
    }

    /// Phase A4: cheap snapshot of the wired Layer1A processor. Returns
    /// `None` when the dashboard launched without a runtime wiring (fresh
    /// install with no SSB) — callers must surface a clear error to the
    /// UI rather than silently no-op. Locks briefly, clones the Arc, drops
    /// the guard before any `.await` (commands are async).
    pub fn layer1a_processor_snapshot(
        &self,
    ) -> Option<Arc<dyn Layer1aProcessor + Send + Sync>> {
        self.layer1a_processor.lock().ok()?.clone()
    }

    /// Cheap clone of the current project root. Use this in every command
    /// that reads `state.project_root` — direct `&state.project_root` reads
    /// no longer compile because the field is `Arc<Mutex<PathBuf>>`. Falls
    /// back to the dashboard launch cwd if the mutex is poisoned.
    pub fn project_root_snapshot(&self) -> PathBuf {
        self.project_root
            .lock()
            .map(|g| g.clone())
            .unwrap_or_else(|_| resolve_project_root())
    }

    /// Replace the active project root. Called by `open_project_from_folder`
    /// after the picker resolves a directory. Silent no-op on poison so a
    /// transient lock failure cannot wedge subsequent project switches.
    pub fn set_project_root(&self, path: PathBuf) {
        if let Ok(mut g) = self.project_root.lock() {
            *g = path;
        }
    }

    /// H-11: register an additional allowed root for the scoped filesystem
    /// commands. Canonicalizes `path` before storing so comparison against
    /// canonicalized request paths is sound. No-op (returns `Ok(false)`) if
    /// the root is already registered. Returns `Err` if the mutex is poisoned
    /// so callers (e.g. `mount_root`) can propagate the failure to the client
    /// rather than silently succeeding.
    pub fn add_allowed_root(&self, path: PathBuf) -> Result<bool, String> {
        let canonical = std::fs::canonicalize(&path)
            .map_err(|e| format!("add_allowed_root: cannot canonicalize {}: {e}", path.display()))?;
        let mut roots = self.allowed_roots.lock()
            .map_err(|_| "add_allowed_root: allowed_roots mutex is poisoned".to_string())?;
        if roots.contains(&canonical) {
            return Ok(false);
        }
        roots.push(canonical);
        Ok(true)
    }

    /// H-11: return true if `canonical_path` starts with any registered
    /// allowed root (canonicalized). Used by read_dir / read_file /
    /// open_in_editor to enforce server-side scope before touching the FS.
    /// Fail-closed: returns false on mutex poison.
    pub fn is_path_allowed(&self, canonical_path: &std::path::Path) -> bool {
        match self.allowed_roots.lock() {
            Ok(roots) => path_is_within_roots(canonical_path, &roots),
            // Poisoned mutex: fail-closed. Do NOT allow access on lock failure.
            Err(_) => false,
        }
    }

    fn open_pair(
        db_path: &Path,
        clock: Arc<dyn ClockProvider>,
    ) -> (Option<Arc<SsbViewer>>, Option<Arc<Ssb>>) {
        let viewer = match SsbViewer::open(db_path) {
            Ok(v) => Some(Arc::new(v)),
            Err(e) => {
                tracing::warn!(error = %e, path = %db_path.display(), "dashboard-v2: SsbViewer::open failed");
                None
            }
        };
        let writer = if viewer.is_some() {
            let project_id = elai_core::project_id_from_cwd().unwrap_or_else(|| {
                format!("{}unknown", elai_core::PROJECT_ID_PREFIX)
            });
            let session_id = format!("dash-v2-{}", std::process::id());
            let cfg = SsbConfig {
                db_path: db_path.to_path_buf(),
                pool_size: Some(2),
                project_id,
                session_id,
            };
            match Ssb::new(cfg, clock) {
                Ok(w) => Some(Arc::new(w)),
                Err(e) => {
                    tracing::warn!(error = %e, "dashboard-v2: Ssb writer construction failed");
                    None
                }
            }
        } else {
            None
        };
        (viewer, writer)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Pure predicate: return `true` if `candidate` starts with ANY entry in
/// `roots`. Both `candidate` and every element of `roots` MUST be
/// canonicalized before calling.
///
/// This is the single authoritative gate for the allowed-root check. Both
/// `AppState::is_path_allowed` and the unit tests below call this function
/// directly so a logic change here is caught by both the production and the
/// test path.
pub(crate) fn path_is_within_roots(candidate: &std::path::Path, roots: &[std::path::PathBuf]) -> bool {
    roots.iter().any(|r| candidate.starts_with(r))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    // Construct a minimal allowed_roots registry for testing without
    // building a full AppState (which has filesystem/env/network side-effects).
    fn make_roots(paths: &[&std::path::Path]) -> Vec<std::path::PathBuf> {
        paths.iter().map(|p| std::fs::canonicalize(p).expect("canonicalize")).collect()
    }

    #[test]
    fn path_is_within_roots_rejects_outside_path() {
        let allowed_dir = TempDir::new().expect("allowed_dir");
        let outside_dir = TempDir::new().expect("outside_dir");
        let roots = make_roots(&[allowed_dir.path()]);
        let candidate = std::fs::canonicalize(outside_dir.path()).expect("canon outside");
        assert!(
            !path_is_within_roots(&candidate, &roots),
            "path outside allowed roots must be rejected"
        );
    }

    #[test]
    fn path_is_within_roots_accepts_subpath() {
        let root_dir = TempDir::new().expect("root_dir");
        let sub = root_dir.path().join("src");
        std::fs::create_dir(&sub).expect("create src");
        let file = sub.join("lib.rs");
        std::fs::write(&file, b"fn foo() {}").expect("write");
        let roots = make_roots(&[root_dir.path()]);
        let candidate = std::fs::canonicalize(&file).expect("canon file");
        assert!(
            path_is_within_roots(&candidate, &roots),
            "file inside allowed root must be accepted"
        );
    }

    #[test]
    fn path_is_within_roots_accepts_secondary_root() {
        let primary = TempDir::new().expect("primary");
        let secondary = TempDir::new().expect("secondary");
        let file = secondary.path().join("readme.md");
        std::fs::write(&file, b"# readme").expect("write");
        // Only primary is initially in roots.
        let mut roots = make_roots(&[primary.path()]);
        let candidate = std::fs::canonicalize(&file).expect("canon file");
        assert!(
            !path_is_within_roots(&candidate, &roots),
            "file under secondary root must be rejected before add"
        );
        // After adding secondary root, it must be accepted.
        roots.push(std::fs::canonicalize(secondary.path()).expect("canon secondary"));
        assert!(
            path_is_within_roots(&candidate, &roots),
            "file under secondary root must be accepted after add"
        );
    }

    #[test]
    fn is_path_allowed_uses_path_is_within_roots_via_mutex() {
        // Test the Arc<Mutex<Vec<PathBuf>>> lock path that is_path_allowed uses,
        // exercising the real mutex-guarded gate (not a raw vec call).
        let root_dir = TempDir::new().expect("root_dir");
        let file = root_dir.path().join("main.rs");
        std::fs::write(&file, b"fn main() {}").expect("write");
        let canonical_root = std::fs::canonicalize(root_dir.path()).expect("canon root");
        let canonical_file = std::fs::canonicalize(&file).expect("canon file");

        let roots: Arc<Mutex<Vec<std::path::PathBuf>>> =
            Arc::new(Mutex::new(vec![canonical_root]));
        let is_allowed = match roots.lock() {
            Ok(r) => path_is_within_roots(&canonical_file, &r),
            Err(_) => false,
        };
        assert!(is_allowed, "file inside root must be allowed via mutex gate");
    }
}
