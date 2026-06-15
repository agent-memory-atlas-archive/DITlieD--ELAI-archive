// Phase B2 — IPC accept loop: orchestrator-side listen/accept/route cycle.
//
// Responsibilities:
//   - bind_session_listener: create platform-native listener (UnixListener on
//     Unix, NamedPipeServer on Windows) before the agent process is spawned.
//   - accept_one: accept a single connection from the listener.
//   - AgentRpcServer trait: abstraction over infer/tool dispatch so tests can
//     inject a ScriptedAgentRpcServer without LLM/tool subsystem coupling.
//   - run_agent_session: read frames in a loop, route via IpcGateway, call
//     the RPC server, write response frames back, return AgentSessionOutcome.
//
// Invariant (listen-before-spawn):
//   The caller MUST call bind_session_listener BEFORE spawning the agent
//   process. This guarantees the socket endpoint exists when the agent's
//   connect() call fires.
//
// Error handling: all fallible paths use IpcAcceptError (thiserror). No
// .unwrap()/.expect()/panic!/todo! outside #[cfg(test)] blocks.
//
// Async I/O: tokio::io::{AsyncReadExt, AsyncWriteExt}. Mutex: tokio::sync.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex as AsyncMutex;
use uuid::Uuid;

use elai_ipc_types::{
    IpcFrame, IpcFrameError, IpcMessageType,
    FRAME_HEADER_SIZE, IPC_PROTOCOL_VERSION, MAX_FRAME_SIZE,
};
use elai_plugins::{HookContext, IpcHookTrait};

use crate::ipc_gateway::{FrameRouteResult, IpcGateway};

// ---------------------------------------------------------------------------
// C-11: hook dispatcher trait alias used by run_agent_session.
// ---------------------------------------------------------------------------

/// Object-safe trait bound for the optional IpcHookDispatcher parameter.
/// We re-alias the upstream plugin trait so external callers can pass
/// `Option<Arc<dyn IpcHookDispatcher + Send + Sync + 'static>>` without
/// importing `elai_plugins` directly.
///
/// C-11 D4: keyed on FrameRouteResult::ToolRequest only; pre_tool_use fires
/// BEFORE rpc.handle_tool_request and post_tool_use fires AFTER, with the
/// returned ToolOutcome bytes stringified. Other variants (InferRequest,
/// BashRequest, GroundingRequest, ThinkStep, Checkpoint, Heartbeat) are
/// out-of-scope for the IPC hook surface (see C-11-decisions.md D4 + D6).
pub trait IpcHookDispatcher: IpcHookTrait {}

impl<T: IpcHookTrait + ?Sized> IpcHookDispatcher for T {}

// ---------------------------------------------------------------------------
// 4C PATH-TRAVERSAL guard — reject ToolRequests whose target path escapes the
// project/session root BEFORE the request reaches pre_tool_use / dispatch.
// ---------------------------------------------------------------------------

/// Env var that overrides the project root used as the path-traversal jail
/// for agent `ToolRequest` frames. When unset, the orchestrator process's
/// current working directory (the canonical ELAI session root) is used.
pub const ELAI_PROJECT_ROOT_ENV: &str = "ELAI_PROJECT_ROOT";

/// Sanitized error returned to the agent when a ToolRequest path escapes the
/// project root. Carries no absolute path so the jail boundary is not leaked.
const PATH_TRAVERSAL_REJECT_MSG: &str = "Tool call rejected: path outside project root";

/// JSON keys, in priority order, that a tool's `arguments` may use to name a
/// filesystem target. Mirrors `permissions::extract_permission_subject` so the
/// IPC boundary and the permission layer agree on what counts as "the path".
const TOOL_PATH_ARG_KEYS: &[&str] = &[
    "path",
    "file_path",
    "filePath",
    "notebook_path",
    "notebookPath",
];

/// Resolve the project root that bounds agent file access. Reads
/// [`ELAI_PROJECT_ROOT_ENV`] first, falling back to the process cwd. Returns
/// `None` only when neither source yields a usable path (cwd unreadable AND
/// env unset), in which case the caller fails closed.
fn resolve_project_root() -> Option<PathBuf> {
    if let Ok(v) = std::env::var(ELAI_PROJECT_ROOT_ENV) {
        if !v.trim().is_empty() {
            return Some(PathBuf::from(v));
        }
    }
    std::env::current_dir().ok()
}

/// Extract the first filesystem-target path from a tool's JSON `arguments`.
/// Returns `None` when the bytes are not UTF-8 JSON, not an object, or carry
/// none of [`TOOL_PATH_ARG_KEYS`] (a tool that names no path cannot traverse).
fn extract_tool_path(arguments: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(arguments).ok()?;
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    let obj = value.as_object()?;
    for key in TOOL_PATH_ARG_KEYS {
        if let Some(s) = obj.get(*key).and_then(serde_json::Value::as_str) {
            if !s.trim().is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// InferOutcome / ToolOutcome
// ---------------------------------------------------------------------------

/// Outcome of an orchestrator-side inference call.
#[derive(Debug)]
pub enum InferOutcome {
    /// LLM returned a text response.
    Response(String),
    /// Inference failed; the reason is returned to the agent as an error frame.
    Failed(String),
}

/// Outcome of an orchestrator-side tool call.
#[derive(Debug)]
pub enum ToolOutcome {
    /// Tool returned serialized output bytes.
    Result(Vec<u8>),
    /// Tool execution failed.
    Failed(String),
}

// ---------------------------------------------------------------------------
// AgentRpcServer trait
// ---------------------------------------------------------------------------

/// Orchestrator-side RPC abstraction for agent-originated requests.
///
/// Implementors decide how to route `InferRequest` (to a sidecar LLM) and
/// `ToolRequest` (to the in-process tool dispatcher). The trait is object-safe
/// so it can be stored as `Arc<dyn AgentRpcServer>`.
///
/// Default implementations log via `tracing` and return
/// `AgentRpcError::Unsupported` for frame variants that are handled
/// structurally by `run_agent_session` (Heartbeat, ThinkStep, Checkpoint)
/// rather than by the RPC layer.
pub trait AgentRpcServer: Send + Sync {
    /// Handle an `InferRequest` from an agent.
    fn handle_infer_request<'a>(
        &'a self,
        prompt: String,
        max_tokens: u32,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = InferOutcome> + Send + 'a>>;

    /// Handle a `ToolRequest` from an agent.
    fn handle_tool_request<'a>(
        &'a self,
        tool_id: String,
        arguments: Vec<u8>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ToolOutcome> + Send + 'a>>;

    /// I-14: deterministic context-compression middleware seam.
    ///
    /// Called by `run_agent_session` on a SUCCESSFUL tool-result's bytes, AFTER
    /// `handle_tool_request` resolves and BEFORE the `ToolResult` frame is written to
    /// the agent. This is the only point where tool output is accessible as raw bytes
    /// before it enters the agent's conversation context (the volatile tail), so it is
    /// the canonical Tier-1 compression hook.
    ///
    /// The default impl is identity (returns `output` unchanged): harness / test RPC
    /// servers ship no compression and no SSB receipt. The production server
    /// (`ProductionAgentRpcServer`) overrides it to run `ContextTransformer::transform`
    /// and journal a reversible `ContextTransformReceipt` SSB row (ID-3).
    ///
    /// `superstep` is forwarded onto the receipt row so the dashboard can correlate the
    /// transform with the superstep it fired in.
    fn transform_tool_output<'a>(
        &'a self,
        output: Vec<u8>,
        _superstep: i64,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<u8>> + Send + 'a>> {
        Box::pin(async move { output })
    }
}

// ---------------------------------------------------------------------------
// AgentSessionOutcome
// ---------------------------------------------------------------------------

/// Outcome returned by `run_agent_session` after the agent exits or times out.
#[derive(Debug)]
pub struct AgentSessionOutcome {
    pub final_state: AgentFinalState,
    pub frames_handled: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AgentFinalState {
    /// Agent sent `AgentResponse` or `AgentMaxIterationsReached` (clean exit).
    Passed,
    /// Agent sent a `Rejected` frame, timed out, or connection was dropped.
    Failed { reason: String },
}

// ---------------------------------------------------------------------------
// PlatformListener / PlatformStream — cfg-split wrappers
// ---------------------------------------------------------------------------

/// Platform-native listener wrapping Unix `UnixListener` or Windows
/// `NamedPipeServer`. Created by `bind_session_listener`; consumed by
/// `accept_one`.
pub enum PlatformListener {
    #[cfg(unix)]
    Unix(tokio::net::UnixListener),
    #[cfg(windows)]
    WindowsPipe {
        /// The named-pipe path (e.g. `\\.\pipe\elai-<uuid>.sock`).
        pipe_path: String,
        /// The live server handle. MUST remain open: on Windows, closing all
        /// server handles destroys the named-pipe endpoint (per MSDN —
        /// https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-instances).
        /// `accept_one` swaps this out for a fresh instance before calling
        /// `connect()`, keeping the endpoint alive across multiple accepts.
        server: tokio::net::windows::named_pipe::NamedPipeServer,
    },
}

impl PlatformListener {
    /// Return the address string a CHILD AGENT must use to connect to this
    /// listener. Critical for cross-platform IPC bring-up:
    ///   - Unix: returns the filesystem socket path as-is
    ///     (e.g. `/tmp/.../agent-<uuid>.sock`).
    ///   - Windows: returns the named-pipe NAME
    ///     (e.g. `\\.\pipe\elai-agent-<uuid>`),
    ///     NOT the filesystem path `C:\...\agent-<uuid>.sock`.
    ///
    /// Pass this string (not the original socket_path) to launch_agent_process
    /// so the agent's connect()/open() call hits the same endpoint the
    /// orchestrator just bound. The agent has no way to compute the
    /// Windows-pipe transformation on its own without duplicating
    /// `socket_path_to_pipe_name` — so the listener returns it directly.
    ///
    /// Bug history (patch-R2.2, 2026-05-17): before this method existed,
    /// launch_agent_process passed `socket_path.to_string_lossy()` for both
    /// platforms. On Windows the agent did `OpenOptions::open(filesystem_path)`
    /// and got ERROR_FILE_NOT_FOUND — accept_one then timed out at 120s with
    /// `exit_code=None`. B3 cloud e2e surfaced this in run 2.
    #[must_use]
    pub fn agent_connect_address(&self) -> String {
        match self {
            #[cfg(unix)]
            Self::Unix(listener) => listener
                .local_addr()
                .ok()
                .and_then(|a| a.as_pathname().map(|p| p.to_string_lossy().into_owned()))
                .unwrap_or_default(),
            #[cfg(windows)]
            Self::WindowsPipe { pipe_path, .. } => pipe_path.clone(),
        }
    }
}

/// Connected platform-native stream ready for async frame I/O.
pub enum PlatformStream {
    #[cfg(unix)]
    Unix(tokio::net::UnixStream),
    #[cfg(windows)]
    WindowsPipe(tokio::net::windows::named_pipe::NamedPipeServer),
}

// AsyncReadExt / AsyncWriteExt forwarding for PlatformStream is handled via
// the read_frame_from_stream / write_frame_to_stream free functions below so
// we avoid implementing AsyncRead/AsyncWrite traits on an enum (which require
// unsafe pin-projection).

// ---------------------------------------------------------------------------
// bind_session_listener
// ---------------------------------------------------------------------------

/// Create and bind a platform-native listener at `socket_path`.
///
/// On Unix: creates a `UnixListener` at the given path (removes any stale
/// socket file first).
/// On Windows: creates a `NamedPipeServer` using the path as the pipe name
/// (e.g. `\\.\pipe\<uuid>.sock`).
///
/// MUST be called before spawning the agent process (listen-before-spawn).
pub async fn bind_session_listener(socket_path: &Path) -> Result<PlatformListener, IpcAcceptError> {
    #[cfg(unix)]
    {
        // Remove stale socket file if present.
        if socket_path.exists() {
            tokio::fs::remove_file(socket_path)
                .await
                .map_err(|e| IpcAcceptError::BindFailed(format!("remove_stale_socket: {e}")))?;
        }
        // Ensure parent directory exists.
        if let Some(parent) = socket_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| IpcAcceptError::BindFailed(format!("create_dir_all: {e}")))?;
        }
        let listener = tokio::net::UnixListener::bind(socket_path)
            .map_err(|e| IpcAcceptError::BindFailed(format!("UnixListener::bind: {e}")))?;
        return Ok(PlatformListener::Unix(listener));
    }

    #[cfg(windows)]
    {
        let _ = socket_path; // used via pipe_path below
        let pipe_path = socket_path_to_pipe_name(socket_path);
        // Create the first server instance with first_pipe_instance(true) and
        // KEEP it alive in PlatformListener. Per MSDN, when all server handles
        // are closed the named-pipe endpoint is destroyed and clients receive
        // ERROR_FILE_NOT_FOUND. Storing the handle here prevents that race.
        let server = tokio::net::windows::named_pipe::ServerOptions::new()
            .first_pipe_instance(true)
            .create(&pipe_path)
            .map_err(|e| IpcAcceptError::BindFailed(format!("NamedPipeServer::create: {e}")))?;
        return Ok(PlatformListener::WindowsPipe { pipe_path, server });
    }

    #[allow(unreachable_code)]
    Err(IpcAcceptError::BindFailed("unsupported platform".to_string()))
}

/// Accept one incoming connection from `listener`.
///
/// On Unix: calls `UnixListener::accept()` and returns the stream.
/// On Windows: creates a fresh `NamedPipeServer` and calls `connect()`.
///
/// Returns a `PlatformStream` ready for frame I/O.
pub async fn accept_one(listener: &mut PlatformListener) -> Result<PlatformStream, IpcAcceptError> {
    match listener {
        #[cfg(unix)]
        PlatformListener::Unix(ul) => {
            let (stream, _addr) = ul
                .accept()
                .await
                .map_err(|e| IpcAcceptError::AcceptFailed(format!("UnixListener::accept: {e}")))?;
            Ok(PlatformStream::Unix(stream))
        }
        #[cfg(windows)]
        PlatformListener::WindowsPipe { pipe_path, server } => {
            // Create the next server instance BEFORE taking the current one.
            // This keeps at least one server handle open at all times, so the
            // pipe endpoint is never destroyed between accepts (MSDN invariant).
            let next_server = tokio::net::windows::named_pipe::ServerOptions::new()
                .create(pipe_path)
                .map_err(|e| {
                    IpcAcceptError::AcceptFailed(format!("NamedPipeServer::create (next): {e}"))
                })?;
            // Swap: replace the stored handle with the fresh one, take the
            // previous handle to drive connect() on.
            let current_server = std::mem::replace(server, next_server);
            current_server
                .connect()
                .await
                .map_err(|e| IpcAcceptError::AcceptFailed(format!("NamedPipeServer::connect: {e}")))?;
            Ok(PlatformStream::WindowsPipe(current_server))
        }
    }
}

// ---------------------------------------------------------------------------
// Async frame I/O helpers
// ---------------------------------------------------------------------------

/// Read one IPC frame from a `PlatformStream`. Returns the decoded message.
pub async fn read_frame_from_stream(
    stream: &mut PlatformStream,
) -> Result<IpcMessageType, IpcAcceptError> {
    match stream {
        #[cfg(unix)]
        PlatformStream::Unix(s) => read_frame_inner(s).await,
        #[cfg(windows)]
        PlatformStream::WindowsPipe(s) => read_frame_inner(s).await,
    }
}

/// Write one IPC message as a framed wire sequence to a `PlatformStream`.
pub async fn write_frame_to_stream(
    stream: &mut PlatformStream,
    msg: &IpcMessageType,
) -> Result<(), IpcAcceptError> {
    match stream {
        #[cfg(unix)]
        PlatformStream::Unix(s) => write_frame_inner(s, msg).await,
        #[cfg(windows)]
        PlatformStream::WindowsPipe(s) => write_frame_inner(s, msg).await,
    }
}

async fn read_frame_inner<R>(reader: &mut R) -> Result<IpcMessageType, IpcAcceptError>
where
    R: AsyncReadExt + Unpin,
{
    let mut header = [0u8; FRAME_HEADER_SIZE];
    reader
        .read_exact(&mut header)
        .await
        .map_err(|e| IpcAcceptError::Io(format!("read_header: {e}")))?;

    let version = header[0];
    if version != IPC_PROTOCOL_VERSION {
        return Err(IpcAcceptError::Frame(IpcFrameError::VersionMismatch {
            expected: IPC_PROTOCOL_VERSION,
            actual: version,
        }));
    }

    let len = u32::from_le_bytes([header[1], header[2], header[3], header[4]]) as usize;
    if len > MAX_FRAME_SIZE {
        return Err(IpcAcceptError::Frame(IpcFrameError::PayloadTooLarge {
            size: len,
            limit: MAX_FRAME_SIZE,
        }));
    }

    let mut payload = vec![0u8; len];
    if len > 0 {
        reader
            .read_exact(&mut payload)
            .await
            .map_err(|e| IpcAcceptError::Io(format!("read_payload: {e}")))?;
    }

    IpcFrame::decode(&payload).map_err(IpcAcceptError::Frame)
}

async fn write_frame_inner<W>(writer: &mut W, msg: &IpcMessageType) -> Result<(), IpcAcceptError>
where
    W: AsyncWriteExt + Unpin,
{
    let frame = IpcFrame::encode(msg).map_err(IpcAcceptError::Frame)?;
    let wire = frame.to_wire_bytes().map_err(IpcAcceptError::Frame)?;
    writer
        .write_all(&wire)
        .await
        .map_err(|e| IpcAcceptError::Io(format!("write_all: {e}")))?;
    writer
        .flush()
        .await
        .map_err(|e| IpcAcceptError::Io(format!("flush: {e}")))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// run_agent_session
// ---------------------------------------------------------------------------

/// Drive a single agent session: read frames in a loop, route via IpcGateway,
/// call the RPC server for InferRequest/ToolRequest, write response frames.
///
/// Returns when:
///   - Agent sends `AgentResponse` or `AgentMaxIterationsReached` (Passed).
///   - A `Rejected` frame is routed (Failed).
///   - `timeout_ms` elapses without a terminal frame (Failed).
///   - The connection is dropped / IO error (Failed).
///
/// The caller is responsible for registering the `AgentConnection` in the
/// gateway BEFORE calling this function, and for removing it afterwards.
pub async fn run_agent_session(
    gateway: Arc<AsyncMutex<IpcGateway>>,
    rpc: Arc<dyn AgentRpcServer + Send + Sync + 'static>,
    agent_id: Uuid,
    mut stream: PlatformStream,
    timeout_ms: u64,
    hook_dispatcher: Option<Arc<dyn IpcHookDispatcher + Send + Sync + 'static>>,
) -> AgentSessionOutcome
{
    // C-15: OsClock-backed deadline (raw Instant arithmetic).
    // `saturating_duration_since` requires a real Instant comparator, so we
    // hold both ends as raw Instants. This stays clean of the entropy scanner
    // because the calls go through `OsClock::instant_now()`.
    let session_clock = OsClock;
    let session_deadline = session_clock.instant_now()
        + std::time::Duration::from_millis(timeout_ms);
    let mut frames_handled: u32 = 0;

    loop {
        // Check overall session timeout.
        let remaining = session_deadline.saturating_duration_since(session_clock.instant_now());
        if remaining.is_zero() {
            return AgentSessionOutcome {
                final_state: AgentFinalState::Failed {
                    reason: format!("session timeout after {timeout_ms}ms"),
                },
                frames_handled,
            };
        }

        // Read one frame with remaining-time bound.
        let msg = match tokio::time::timeout(remaining, read_frame_from_stream(&mut stream)).await {
            Ok(Ok(m)) => m,
            Ok(Err(e)) => {
                return AgentSessionOutcome {
                    final_state: AgentFinalState::Failed {
                        reason: format!("frame read error: {e}"),
                    },
                    frames_handled,
                };
            }
            Err(_) => {
                // phase-3C.2: the per-frame read bound just elapsed. Ask the
                // gateway whether THIS agent's connection went silent on
                // heartbeats (the periodic `check_heartbeat_timeouts()` poll
                // the spec calls for — the read deadline IS the poll cadence).
                // If so, stamp HEARTBEAT_TIMEOUT_MARKER so the downstream
                // recovery path classifies it as `Timeout` (wedged agent)
                // instead of `Crash`. Otherwise keep the generic read-timeout
                // reason (a slow-but-live agent that never hung).
                let heartbeat_timed_out = {
                    let gw = gateway.lock().await;
                    gw.check_heartbeat_timeouts().contains(&agent_id)
                };
                let reason = if heartbeat_timed_out {
                    format!(
                        "{} read timed out after {timeout_ms}ms",
                        crate::recovery::HEARTBEAT_TIMEOUT_MARKER
                    )
                } else {
                    format!("read timed out after {timeout_ms}ms")
                };
                return AgentSessionOutcome {
                    final_state: AgentFinalState::Failed { reason },
                    frames_handled,
                };
            }
        };

        // Route the frame through IpcGateway.
        let route_result = {
            let mut gw = gateway.lock().await;
            gw.route_frame(&agent_id, msg)
        };

        frames_handled += 1;

        match route_result {
            FrameRouteResult::Response(inner) => {
                // Terminal frame: agent finished its task.
                tracing::info!(agent_id = %agent_id, "agent session complete (AgentResponse)");
                let _ = inner; // response content logged if needed by caller
                return AgentSessionOutcome {
                    final_state: AgentFinalState::Passed,
                    frames_handled,
                };
            }

            FrameRouteResult::Heartbeat => {
                tracing::trace!(agent_id = %agent_id, "heartbeat");
                // No response needed; loop.
            }

            FrameRouteResult::ThinkStep { topic, step_type, over_budget } => {
                if over_budget {
                    tracing::warn!(
                        agent_id = %agent_id,
                        topic = %topic,
                        step_type = %step_type,
                        "GRC budget exceeded (advisory)"
                    );
                } else {
                    tracing::trace!(
                        agent_id = %agent_id,
                        topic = %topic,
                        step_type = %step_type,
                        "think step"
                    );
                }
                // No response; loop.
            }

            FrameRouteResult::Checkpoint { sequence } => {
                tracing::debug!(agent_id = %agent_id, sequence = sequence, "checkpoint");
                // No response; loop.
            }

            FrameRouteResult::InferRequest { prompt, max_tokens } => {
                let outcome = rpc.handle_infer_request(prompt, max_tokens).await;
                let response_msg = match outcome {
                    InferOutcome::Response(text) => IpcMessageType::InferResponse {
                        text,
                        tokens_used: 0,
                    },
                    InferOutcome::Failed(reason) => {
                        tracing::warn!(
                            agent_id = %agent_id,
                            reason = %reason,
                            "InferRequest failed; sending empty response"
                        );
                        IpcMessageType::InferResponse {
                            text: String::new(),
                            tokens_used: 0,
                        }
                    }
                };
                if let Err(e) = write_frame_to_stream(&mut stream, &response_msg).await {
                    return AgentSessionOutcome {
                        final_state: AgentFinalState::Failed {
                            reason: format!("write InferResponse failed: {e}"),
                        },
                        frames_handled,
                    };
                }
            }

            FrameRouteResult::ToolRequest { tool_id, arguments } => {
                // 4C PATH-TRAVERSAL: reject BEFORE pre_tool_use / dispatch when
                // the tool names a path that canonicalises outside the project
                // root. canonicalise_for_check collapses `..`/`.`/symlinks and
                // tolerates yet-to-be-created write targets. Fail closed: if the
                // root cannot be resolved at all, deny any path-bearing request.
                let traversal_reject: Option<String> = match extract_tool_path(&arguments) {
                    Some(path_str) => match resolve_project_root() {
                        Some(root) => {
                            if crate::sandbox_policy::path_escapes_root(
                                Path::new(&path_str),
                                &root,
                            ) {
                                tracing::warn!(
                                    agent_id = %agent_id,
                                    tool_id = %tool_id,
                                    "ToolRequest rejected: path escapes project root"
                                );
                                Some(PATH_TRAVERSAL_REJECT_MSG.to_string())
                            } else {
                                None
                            }
                        }
                        None => {
                            tracing::error!(
                                agent_id = %agent_id,
                                tool_id = %tool_id,
                                "ToolRequest path check failed closed: project root unresolved"
                            );
                            Some(PATH_TRAVERSAL_REJECT_MSG.to_string())
                        }
                    },
                    // No path argument => nothing to jail here; downstream hooks
                    // and the dispatcher still apply their own checks.
                    None => None,
                };

                // C-11 D4: pre_tool_use hook fires BEFORE rpc dispatch when a
                // dispatcher is wired. BLOCK verdict short-circuits the call
                // and returns a sanitized error to the agent (no rule_id /
                // pattern text leaked — see HookResult::sanitized_agent_message).
                let pre_ctx = HookContext {
                    agent_id: agent_id.to_string(),
                    agent_role: String::new(),
                    session_id: String::new(),
                    fsm_state: "executing".to_string(),
                    tool_id: Some(tool_id.clone()),
                    tool_arguments: Some(arguments.clone()),
                    source_content: None,
                    file_path: None,
                    active_rules_count: 0,
                };
                let blocked_msg = if let Some(reason) = traversal_reject {
                    // Path-traversal denial takes precedence and skips the hook
                    // dispatch entirely — the request never reaches the tool.
                    Some(reason)
                } else if let Some(d) = hook_dispatcher.as_ref() {
                    match d.pre_tool_use(&pre_ctx) {
                        Ok(result) if result.verdict.is_block() => {
                            Some(result.sanitized_agent_message())
                        }
                        Ok(_) => None,
                        Err(e) => {
                            tracing::warn!(
                                agent_id = %agent_id,
                                tool_id = %tool_id,
                                error = %e,
                                "pre_tool_use hook errored; passing through"
                            );
                            None
                        }
                    }
                } else {
                    None
                };

                let (output, success) = if let Some(reason) = blocked_msg {
                    (reason.into_bytes(), false)
                } else {
                    let outcome = rpc.handle_tool_request(tool_id.clone(), arguments).await;
                    let (out, ok) = match outcome {
                        ToolOutcome::Result(bytes) => (bytes, true),
                        ToolOutcome::Failed(reason) => {
                            tracing::warn!(
                                agent_id = %agent_id,
                                tool_id = %tool_id,
                                reason = %reason,
                                "ToolRequest failed"
                            );
                            (reason.into_bytes(), false)
                        }
                    };
                    // C-11 D4: post_tool_use fires AFTER rpc dispatch when a
                    // dispatcher is wired. PostToolUse never short-circuits
                    // (the call already ran); we log BLOCK/WARN but ship the
                    // result bytes to the agent unchanged.
                    if let Some(d) = hook_dispatcher.as_ref() {
                        let post_ctx = HookContext {
                            agent_id: agent_id.to_string(),
                            agent_role: String::new(),
                            session_id: String::new(),
                            fsm_state: "executing".to_string(),
                            tool_id: Some(tool_id.clone()),
                            tool_arguments: None,
                            source_content: None,
                            file_path: None,
                            active_rules_count: 0,
                        };
                        let tool_output_str = std::str::from_utf8(&out)
                            .unwrap_or("");
                        if let Err(e) = d.post_tool_use(&post_ctx, tool_output_str) {
                            tracing::warn!(
                                agent_id = %agent_id,
                                tool_id = %tool_id,
                                error = %e,
                                "post_tool_use hook errored; result returned anyway"
                            );
                        }
                    }
                    // I-14: run the deterministic context-compression middleware on a
                    // SUCCESSFUL tool result before it enters the agent context window.
                    // The production RPC override transforms + journals a receipt; the
                    // default (test/harness) impl is identity. Only successful results
                    // are compressed -- a failure reason string is passed through so the
                    // agent sees the error verbatim.
                    // WIRE-DARK[I-14]: superstep=0 is a placeholder. The real superstep
                    // counter requires threading an i64 through run_agent_session's call
                    // chain; that plumbing lands when the receipt-correlator dashboard
                    // tile (I-14 promotion) is wired. Until then, receipts always record
                    // superstep=0, which is correct for identity receipts (no transform fired).
                    let out = if ok {
                        rpc.transform_tool_output(out, 0).await
                    } else {
                        out
                    };
                    (out, ok)
                };
                let response_msg = IpcMessageType::ToolResult {
                    tool_id: elai_ipc_types::ToolId::new(tool_id),
                    output,
                    success,
                };
                if let Err(e) = write_frame_to_stream(&mut stream, &response_msg).await {
                    return AgentSessionOutcome {
                        final_state: AgentFinalState::Failed {
                            reason: format!("write ToolResult failed: {e}"),
                        },
                        frames_handled,
                    };
                }
            }

            FrameRouteResult::BashRequest { command, timeout_ms: cmd_timeout } => {
                tracing::warn!(
                    agent_id = %agent_id,
                    command = %command,
                    timeout_ms = cmd_timeout,
                    "BashRequest received but not wired (Unsupported); sending empty BashResult"
                );
                let response_msg = IpcMessageType::BashResult {
                    stdout: Vec::new(),
                    stderr: b"BashRequest not wired in ipc_accept".to_vec(),
                    exit_code: 1,
                };
                if let Err(e) = write_frame_to_stream(&mut stream, &response_msg).await {
                    return AgentSessionOutcome {
                        final_state: AgentFinalState::Failed {
                            reason: format!("write BashResult failed: {e}"),
                        },
                        frames_handled,
                    };
                }
            }

            FrameRouteResult::GroundingRequest { topic, query } => {
                tracing::warn!(
                    agent_id = %agent_id,
                    topic = %topic,
                    query = %query,
                    "GroundingRequest received but not wired (Unsupported); sending empty result"
                );
                let response_msg = IpcMessageType::GroundingResult {
                    topic,
                    evidence: Vec::new(),
                    confidence: 0.0,
                };
                if let Err(e) = write_frame_to_stream(&mut stream, &response_msg).await {
                    return AgentSessionOutcome {
                        final_state: AgentFinalState::Failed {
                            reason: format!("write GroundingResult failed: {e}"),
                        },
                        frames_handled,
                    };
                }
            }

            FrameRouteResult::ReplanRequest { reason, severity } => {
                tracing::warn!(
                    agent_id = %agent_id,
                    reason = %reason,
                    severity = %severity,
                    "ReplanRequest received (Unsupported in ipc_accept; treating as session failure)"
                );
                return AgentSessionOutcome {
                    final_state: AgentFinalState::Failed {
                        reason: format!("ReplanRequest: {reason} (severity={severity})"),
                    },
                    frames_handled,
                };
            }

            FrameRouteResult::Rejected(reason) => {
                tracing::error!(
                    agent_id = %agent_id,
                    reason = %reason,
                    "frame rejected by gateway"
                );
                return AgentSessionOutcome {
                    final_state: AgentFinalState::Failed {
                        reason: format!("rejected: {reason}"),
                    },
                    frames_handled,
                };
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Windows pipe name helper
// ---------------------------------------------------------------------------

#[cfg(windows)]
fn socket_path_to_pipe_name(path: &Path) -> String {
    // Convert an arbitrary path like `C:\...\<uuid>.sock` to a valid Windows
    // named-pipe name `\\.\pipe\elai-<uuid>`. The pipe namespace is flat, so
    // we use only the file stem.
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "elai-ipc".to_string());
    format!(r"\\.\pipe\elai-{stem}")
}

// ---------------------------------------------------------------------------
// NoOpAgentRpcServer
// ---------------------------------------------------------------------------

/// Placeholder `AgentRpcServer` that returns `InferOutcome::Failed("unsupported")`
/// and `ToolOutcome::Failed("unsupported")` for all requests. Used by CLI and
/// dashboard call sites until B2c wires the real LLM + tool dispatch.
///
/// No `.unwrap()` / `panic!` / `todo!()` — all methods return graceful
/// failure outcomes that `run_agent_session` treats as failed frames.
pub struct NoOpAgentRpcServer;

impl AgentRpcServer for NoOpAgentRpcServer {
    fn handle_infer_request<'a>(
        &'a self,
        _prompt: String,
        _max_tokens: u32,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = InferOutcome> + Send + 'a>> {
        Box::pin(async move {
            tracing::warn!("NoOpAgentRpcServer: InferRequest received but not wired (B2c pending)");
            InferOutcome::Failed("InferRequest not wired: NoOpAgentRpcServer (B2c pending)".to_string())
        })
    }

    fn handle_tool_request<'a>(
        &'a self,
        _tool_id: String,
        _arguments: Vec<u8>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ToolOutcome> + Send + 'a>> {
        Box::pin(async move {
            tracing::warn!("NoOpAgentRpcServer: ToolRequest received but not wired (B2c pending)");
            ToolOutcome::Failed("ToolRequest not wired: NoOpAgentRpcServer (B2c pending)".to_string())
        })
    }
}

// ---------------------------------------------------------------------------
// test_support — ScriptedAgentRpcServer (cfg-gated)
// ---------------------------------------------------------------------------

/// Test-support helpers for contract tests that need a canned `AgentRpcServer`.
/// Gated on `#[cfg(any(test, feature = "test-support"))]` so this code never
/// compiles into production builds.
#[cfg(any(test, feature = "test-support"))]
pub mod test_support {
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::{AgentRpcServer, InferOutcome, ToolOutcome};

    /// Scripted RPC server that records call counts and returns canned responses.
    /// Used by `ipc_accept_loop_contract` and `agent_pool_listen_before_spawn_contract`.
    pub struct ScriptedAgentRpcServer {
        pub infer_calls: AtomicU32,
        pub tool_calls: AtomicU32,
        /// Canned text returned for every `InferRequest`.
        pub canned_infer_text: String,
    }

    impl ScriptedAgentRpcServer {
        pub fn new(canned_infer_text: impl Into<String>) -> Self {
            Self {
                infer_calls: AtomicU32::new(0),
                tool_calls: AtomicU32::new(0),
                canned_infer_text: canned_infer_text.into(),
            }
        }
    }

    impl AgentRpcServer for ScriptedAgentRpcServer {
        fn handle_infer_request<'a>(
            &'a self,
            _prompt: String,
            _max_tokens: u32,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = InferOutcome> + Send + 'a>> {
            Box::pin(async move {
                self.infer_calls.fetch_add(1, Ordering::Relaxed);
                InferOutcome::Response(self.canned_infer_text.clone())
            })
        }

        fn handle_tool_request<'a>(
            &'a self,
            _tool_id: String,
            _arguments: Vec<u8>,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ToolOutcome> + Send + 'a>> {
            Box::pin(async move {
                self.tool_calls.fetch_add(1, Ordering::Relaxed);
                ToolOutcome::Result(b"scripted_ok".to_vec())
            })
        }
    }
}

// ---------------------------------------------------------------------------
// IpcAcceptError
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum IpcAcceptError {
    #[error("bind failed: {0}")]
    BindFailed(String),

    #[error("accept failed: {0}")]
    AcceptFailed(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("frame error: {0}")]
    Frame(#[from] IpcFrameError),
}

impl std::fmt::Display for AgentFinalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentFinalState::Passed => write!(f, "Passed"),
            AgentFinalState::Failed { reason } => write!(f, "Failed({reason})"),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopRpcServer;
    impl AgentRpcServer for NoopRpcServer {
        fn handle_infer_request<'a>(
            &'a self,
            _prompt: String,
            _max_tokens: u32,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = InferOutcome> + Send + 'a>> {
            Box::pin(async move { InferOutcome::Response("ok".to_string()) })
        }

        fn handle_tool_request<'a>(
            &'a self,
            _tool_id: String,
            _arguments: Vec<u8>,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ToolOutcome> + Send + 'a>> {
            Box::pin(async move { ToolOutcome::Result(b"ok".to_vec()) })
        }
    }

    #[test]
    fn infer_outcome_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<InferOutcome>();
        assert_send_sync::<ToolOutcome>();
    }

    #[test]
    fn agent_rpc_server_is_object_safe() {
        let _: Arc<dyn AgentRpcServer> = Arc::new(NoopRpcServer);
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn bind_and_accept_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("test.sock");
        let mut listener = bind_session_listener(&socket_path).await.unwrap();
        assert!(socket_path.exists());

        // Spawn a connector task.
        let path_clone = socket_path.clone();
        let connector = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            tokio::net::UnixStream::connect(&path_clone).await.unwrap();
        });

        let stream = accept_one(&mut listener).await;
        assert!(stream.is_ok(), "accept_one must succeed");
        connector.await.unwrap();
    }
}
