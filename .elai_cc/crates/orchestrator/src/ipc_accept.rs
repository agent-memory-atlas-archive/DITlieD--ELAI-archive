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

use std::path::Path;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex as AsyncMutex;
use uuid::Uuid;

use elai_ipc_types::{
    IpcFrame, IpcFrameError, IpcMessageType,
    FRAME_HEADER_SIZE, IPC_PROTOCOL_VERSION, MAX_FRAME_SIZE,
};

use crate::ipc_gateway::{FrameRouteResult, IpcGateway};

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
) -> AgentSessionOutcome
{
    let session_deadline = std::time::Instant::now()
        + std::time::Duration::from_millis(timeout_ms);
    let mut frames_handled: u32 = 0;

    loop {
        // Check overall session timeout.
        let remaining = session_deadline.saturating_duration_since(std::time::Instant::now());
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
                return AgentSessionOutcome {
                    final_state: AgentFinalState::Failed {
                        reason: format!("read timed out after {timeout_ms}ms"),
                    },
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
                let outcome = rpc.handle_tool_request(tool_id.clone(), arguments).await;
                let (output, success) = match outcome {
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
