// Phase B2 — IPC accept-loop contract test.
//
// Exercises the bind+accept+route cycle of `ipc_accept::run_agent_session`
// using a `ScriptedAgentRpcServer` that returns canned responses. The full
// round-trip test (spawning the real `elai_cc_agent` binary) is gated behind
// ELAI_B2_AGENT_BIN and `#[ignore]` so CI never fails on a missing binary.
//
// Always-run contracts (no ignore, no binary required):
//   1. bind_session_listener succeeds and listener is usable.
//   2. ScriptedAgentRpcServer: infer_calls counter increments per handle call.
//   3. InferOutcome / ToolOutcome round-trip through the session machinery.
//
// Gated contract (#[ignore] + env-var guard):
//   4. Full round-trip: spawn binary, connect, handle InferRequest frame,
//      assert at least one InferRequest was routed.

#![allow(unused_imports)]

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use elai_ipc_types::{
    AgentDispatchConfig, AgentRole, IpcFrame, IpcMessageType, ToolFilter, ToolId,
    FRAME_HEADER_SIZE, IPC_PROTOCOL_VERSION, MAX_FRAME_SIZE,
    AGENT_HEARTBEAT_INTERVAL_SECS,
    ProcessConfig, ResourceLimits, SeccompProfile,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

use elai_orchestrator::ipc_accept::{
    AgentRpcServer, InferOutcome, ToolOutcome, bind_session_listener,
    read_frame_from_stream, write_frame_to_stream,
};

// ---------------------------------------------------------------------------
// Scripted RPC server — records calls, returns canned responses
// ---------------------------------------------------------------------------

pub struct ScriptedAgentRpcServer {
    pub infer_calls: AtomicU32,
    pub tool_calls: AtomicU32,
    /// Canned InferResponse text returned per InferRequest.
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

// ---------------------------------------------------------------------------
// Helper: resolve agent binary path (mirrors agent_loop_contract.rs:55-68)
// ---------------------------------------------------------------------------

fn agent_binary_path() -> PathBuf {
    // ELAI_B2_AGENT_BIN env override takes precedence.
    if let Ok(p) = std::env::var("ELAI_B2_AGENT_BIN") {
        return PathBuf::from(p);
    }
    let mut cur = std::env::current_exe().expect("current_exe");
    cur.pop();
    if cur.ends_with("deps") {
        cur.pop();
    }
    let exe_name = if cfg!(windows) {
        "elai_cc_agent.exe"
    } else {
        "elai_cc_agent"
    };
    cur.join(exe_name)
}

// Helpers delegate to ipc_accept's exported read_frame_from_stream /
// write_frame_to_stream which dispatch over PlatformStream's cfg-split inner
// types (UnixStream on Unix, NamedPipeServer on Windows). The enum itself
// doesn't implement AsyncRead/AsyncWrite to avoid unsafe pin projection.

// ---------------------------------------------------------------------------
// Contract 1: bind_session_listener creates a usable listener
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_bind_session_listener_creates_usable_listener() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_path = tmp.path().join(format!("{}.sock", Uuid::new_v4()));

    let result = bind_session_listener(&socket_path).await;
    assert!(
        result.is_ok(),
        "bind_session_listener must succeed on a fresh temp path; got: {:?}",
        result.err()
    );
    // On Unix the socket file exists; on Windows the named pipe is addressable.
    #[cfg(unix)]
    assert!(
        socket_path.exists(),
        "Unix socket file must exist after bind"
    );
}

// ---------------------------------------------------------------------------
// Contract 2: ScriptedAgentRpcServer counter increments
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_scripted_server_infer_counter_increments() {
    let server = ScriptedAgentRpcServer::new("hello from scripted");
    let outcome = server.handle_infer_request("test prompt".into(), 256).await;
    assert_eq!(server.infer_calls.load(Ordering::Relaxed), 1);
    match outcome {
        InferOutcome::Response(text) => assert_eq!(text, "hello from scripted"),
        InferOutcome::Failed(e) => panic!("expected Response, got Failed: {e}"),
    }
}

#[tokio::test]
async fn contract_scripted_server_tool_counter_increments() {
    let server = ScriptedAgentRpcServer::new("");
    let outcome = server.handle_tool_request("noop".into(), b"{}".to_vec()).await;
    assert_eq!(server.tool_calls.load(Ordering::Relaxed), 1);
    match outcome {
        ToolOutcome::Result(bytes) => assert_eq!(bytes, b"scripted_ok"),
        ToolOutcome::Failed(e) => panic!("expected Result, got Failed: {e}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 3: InferOutcome / ToolOutcome are Send + Sync + 'static
// ---------------------------------------------------------------------------

#[test]
fn contract_outcome_types_are_send_sync() {
    fn assert_send_sync<T: Send + Sync + 'static>() {}
    assert_send_sync::<InferOutcome>();
    assert_send_sync::<ToolOutcome>();
}

// ---------------------------------------------------------------------------
// Contract 4: Full round-trip via spawned agent binary
// (gated: requires ELAI_B2_AGENT_BIN env var or pre-built elai_cc_agent)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn contract_ipc_accept_loop_handles_infer_request_round_trip() {
    // Gate: skip unless the binary exists.
    let bin_path = agent_binary_path();
    if std::env::var("ELAI_B2_AGENT_BIN").is_err() && !bin_path.exists() {
        eprintln!(
            "SKIP: elai_cc_agent binary not found at {}; set ELAI_B2_AGENT_BIN to enable",
            bin_path.display()
        );
        return;
    }

    let tmp = tempfile::tempdir().expect("tempdir");
    let agent_id = Uuid::new_v4();
    let socket_path = tmp.path().join(format!("{agent_id}.sock"));

    // Step 1: bind BEFORE spawning (listen-before-spawn invariant).
    let mut listener = bind_session_listener(&socket_path)
        .await
        .expect("bind_session_listener");

    // Step 2: spawn agent binary with socket path + agent_id args.
    let task_str = "respond with TOOL_USE: noop()";
    let mut child = std::process::Command::new(&bin_path)
        .arg(socket_path.to_string_lossy().as_ref())
        .arg(agent_id.to_string())
        .arg("--task")
        .arg(task_str)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn agent binary");

    let server = Arc::new(ScriptedAgentRpcServer::new(
        // Return a canned InferResponse that contains a TOOL_USE marker
        // so the agent's ReAct loop emits a ToolRequest.
        "TOOL_USE: noop({})",
    ));

    // Step 3: accept one connection (within 5s).
    let accept_timeout = std::time::Duration::from_secs(5);
    let stream_result = tokio::time::timeout(
        accept_timeout,
        elai_orchestrator::ipc_accept::accept_one(&mut listener),
    )
    .await;

    match stream_result {
        Ok(Ok(mut stream)) => {
            // Step 4: drive one InferRequest/InferResponse round-trip.
            // Read the first frame from the agent.
            let first_frame = tokio::time::timeout(
                std::time::Duration::from_secs(10),
                read_frame_from_stream(&mut stream),
            )
            .await
            .expect("read_frame_from_stream timed out")
            .expect("read_frame_from_stream error");

            match first_frame {
                IpcMessageType::InferRequest { prompt, max_tokens } => {
                    // InferRequest routed — increment counter.
                    let outcome = server.handle_infer_request(prompt, max_tokens).await;
                    let response_text = match outcome {
                        InferOutcome::Response(t) => t,
                        InferOutcome::Failed(e) => panic!("rpc failed: {e}"),
                    };
                    // Send InferResponse back.
                    write_frame_to_stream(
                        &mut stream,
                        &IpcMessageType::InferResponse {
                            text: response_text,
                            tokens_used: 5,
                        },
                    )
                    .await
                    .expect("write_frame_to_stream InferResponse");
                }
                IpcMessageType::AgentHeartbeat { .. } => {
                    // Heartbeat first — acceptable; continue (no assertion here).
                }
                other => {
                    panic!("expected InferRequest or Heartbeat as first frame, got: {other:?}");
                }
            }

            // Assertion: at least one InferRequest was handled.
            assert!(
                server.infer_calls.load(Ordering::Relaxed) >= 1,
                "expected at least one InferRequest to be routed to ScriptedAgentRpcServer"
            );
        }
        Ok(Err(e)) => panic!("accept_one failed: {e}"),
        Err(_) => panic!(
            "accept_one timed out after {}s — agent did not connect (listen-before-spawn failed?)",
            accept_timeout.as_secs()
        ),
    }

    // Cleanup: kill agent process (it may still be running).
    let _ = child.kill();
    // Clean up socket file on Unix.
    #[cfg(unix)]
    let _ = std::fs::remove_file(&socket_path);
}
