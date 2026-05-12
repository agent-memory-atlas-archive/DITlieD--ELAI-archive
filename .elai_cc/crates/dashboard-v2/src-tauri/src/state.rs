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

use std::collections::{BTreeMap, HashMap};
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

/// Top-level entry surfaced by `get_media_validations`. One per `step_id`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MediaValidationEntry {
    /// Step id from the originating pipeline step. Used as the HashMap key.
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
use elai_orchestrator::{
    DefaultFeasibilityProber, FeasibilityProber, Ssb, SsbConfig, SsbViewer,
};
use elai_web_validator::DevServerHandle;

/// Default SSB database path resolution. We look in this order:
///   1. `ELAI_SSB_PATH` env var (explicit override).
///   2. `elai_core::paths::sessions_dir_path(&cwd)/ssb.db` (canonical: `.elai_cc/sessions/ssb.db`).
///   3. `<cwd>/.elai/sessions/ssb.db` (legacy path, intentional fallback per B-1 L94-98).
/// Returns `None` if no candidate exists.
pub(crate) fn resolve_ssb_path() -> Option<PathBuf> {
    if let Ok(env_path) = std::env::var("ELAI_SSB_PATH") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return Some(p);
        }
    }
    let cwd = std::env::current_dir().ok()?;
    let candidates = [
        sessions_dir_path(&cwd).join("ssb.db"),              // canonical
        cwd.join(".elai").join("sessions").join("ssb.db"),   // legacy, intentional fallback (B-1 L94-98)
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
    pub media_validation: Arc<tokio::sync::RwLock<HashMap<String, MediaValidationEntry>>>,
}

impl AppState {
    pub fn new() -> Self {
        let project_root = resolve_project_root();
        let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);

        let (viewer, writer) = match resolve_ssb_path() {
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
        let feasibility_root = project_root.join(".elai_cc").join("feasibility");
        let feasibility_prober: Arc<dyn FeasibilityProber> =
            Arc::new(DefaultFeasibilityProber::new(feasibility_root));

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
            media_validation: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
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
