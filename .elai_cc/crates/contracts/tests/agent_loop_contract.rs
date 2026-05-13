// Phase A2 contract — agent-side ReAct loop in crates/agent.
//
// Pins the two acceptance criteria from workflow/handoff/phase-A2.md:
//
//   Test 1 (happy_path_two_iterations):
//     - spawn elai_cc_agent binary via a bound IPC socket / named-pipe
//       fixture (mirrors the Tier3StubExecutor pattern at
//       feasibility_probe.rs:498).
//     - send AgentRequest{task} -> expect InferRequest from agent.
//     - reply with InferResponse whose `text` contains one TOOL_USE marker.
//     - expect ToolRequest{tool_id} -> reply ToolResult{output=b"42",ok}.
//     - expect a SECOND InferRequest (proves the tool result was threaded
//       back into the conversation). The second prompt MUST contain the
//       tool_id from the previous round (proves round-trip).
//     - reply with InferResponse text containing NO TOOL_USE marker.
//     - expect AgentResponse{content non-empty}.
//     - assert: iteration count = 2; tool_id round-tripped.
//
//   Test 2 (max_iterations_cap):
//     - reply to every InferRequest with one TOOL_USE marker.
//     - after 25 ToolRequest -> ToolResult round-trips, the 26th iteration
//       cap fires -> agent emits IpcMessageType::AgentMaxIterationsReached
//       { reason, iterations_completed: 25 }.
//     - assert: NO AgentResponse follows.
//
// Tool-use protocol: the test injects a literal text marker
// `TOOL_USE: <tool_id>` inside the InferResponse.text field. The agent's
// run_react_turn parses the flat text for those lines; presence triggers a
// ToolRequest, absence is the end_turn signal. This is the deviation
// documented in workflow/handoff/design-deviations.md (the IPC InferResponse
// shape is a flat String, not a Vec<ContentBlock>; the agent crate has no
// access to the orchestrator-side ContentBlock type by design).

#![allow(clippy::too_many_lines)]

use std::path::PathBuf;
use std::time::Duration;

use elai_ipc_types::{
    AgentDispatchConfig, AgentRole, IpcFrame, IpcMessageType, ProcessConfig, ResourceLimits,
    SeccompProfile, ToolFilter, ToolId, FRAME_HEADER_SIZE, IPC_PROTOCOL_VERSION, MAX_FRAME_SIZE,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Binary path resolution — locate the workspace-built elai_cc_agent binary.
// ---------------------------------------------------------------------------
//
// The integration-test runner lives at `target/debug/deps/elai_cc_contracts-<hash>`.
// The agent binary lives at `target/debug/elai_cc_agent[.exe]`. Walk up from
// the current_exe() until we find a directory whose `parent()` is named
// "debug" (or any profile directory), then resolve the sibling binary.

fn agent_binary_path() -> PathBuf {
    let mut cur = std::env::current_exe().expect("current_exe");
    // Pop the test binary filename and the `deps` directory to reach the
    // profile dir (e.g. target/debug or target/release).
    cur.pop(); // deps/<test_binary>
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

// ---------------------------------------------------------------------------
// IPC fixture helpers — async frame I/O mirroring feasibility_probe.rs.
// ---------------------------------------------------------------------------

async fn write_frame<W>(writer: &mut W, msg: &IpcMessageType) -> Result<(), String>
where
    W: AsyncWriteExt + Unpin,
{
    let frame = IpcFrame::encode(msg).map_err(|e| format!("ipc_encode:{e}"))?;
    let bytes = frame.to_wire_bytes().map_err(|e| format!("ipc_wire:{e}"))?;
    writer
        .write_all(&bytes)
        .await
        .map_err(|e| format!("ipc_write:{e}"))?;
    writer.flush().await.map_err(|e| format!("ipc_flush:{e}"))?;
    Ok(())
}

async fn read_frame<R>(reader: &mut R) -> Result<IpcMessageType, String>
where
    R: AsyncReadExt + Unpin,
{
    let mut header = [0u8; FRAME_HEADER_SIZE];
    reader
        .read_exact(&mut header)
        .await
        .map_err(|e| format!("ipc_read_header:{e}"))?;
    let version = header[0];
    if version != IPC_PROTOCOL_VERSION {
        return Err(format!(
            "ipc_version_mismatch:expected={IPC_PROTOCOL_VERSION},got={version}"
        ));
    }
    let len = u32::from_le_bytes([header[1], header[2], header[3], header[4]]) as usize;
    if len > MAX_FRAME_SIZE {
        return Err(format!("ipc_payload_too_large:{len}"));
    }
    let mut payload = vec![0u8; len];
    reader
        .read_exact(&mut payload)
        .await
        .map_err(|e| format!("ipc_read_payload:{e}"))?;
    IpcFrame::decode(&payload).map_err(|e| format!("ipc_decode:{e}"))
}

fn default_dispatch_config() -> AgentDispatchConfig {
    AgentDispatchConfig {
        role: AgentRole::Executor,
        model_preference: "qwen2.5-coder".to_owned(),
        tool_filter: ToolFilter {
            allowed_tools: vec![ToolId("echo".to_owned())],
        },
        rule_domains: vec!["test".to_owned()],
        grc_budget_max_calls: 8,
        grc_budget_max_chars: 1500,
        sandbox_config: ProcessConfig {
            allowed_paths: Vec::new(),
            seccomp_profile: SeccompProfile::IpcBaseline,
            resource_limits: ResourceLimits {
                max_memory_bytes: 512 * 1024 * 1024,
                max_cpu_secs: 60,
                max_open_files: 32,
            },
        },
    }
}

// ---------------------------------------------------------------------------
// Platform-specific socket / pipe binding.
// ---------------------------------------------------------------------------

#[cfg(unix)]
async fn bind_endpoint(socket_id: &Uuid) -> (tokio::net::UnixListener, PathBuf) {
    let path = std::env::temp_dir().join(format!("elai_a2_{socket_id}.sock"));
    let _ = tokio::fs::remove_file(&path).await;
    let listener = tokio::net::UnixListener::bind(&path).expect("bind unix socket");
    (listener, path)
}

#[cfg(unix)]
async fn accept_endpoint(listener: tokio::net::UnixListener) -> tokio::net::UnixStream {
    let (stream, _addr) = listener.accept().await.expect("accept");
    stream
}

#[cfg(windows)]
async fn bind_endpoint(
    socket_id: &Uuid,
) -> (tokio::net::windows::named_pipe::NamedPipeServer, PathBuf) {
    let pipe_name = format!(r"\\.\pipe\elai_a2_{socket_id}");
    let server = tokio::net::windows::named_pipe::ServerOptions::new()
        .first_pipe_instance(true)
        .create(&pipe_name)
        .expect("create named pipe");
    (server, PathBuf::from(pipe_name))
}

#[cfg(windows)]
async fn accept_endpoint(
    server: tokio::net::windows::named_pipe::NamedPipeServer,
) -> tokio::net::windows::named_pipe::NamedPipeServer {
    server.connect().await.expect("accept named pipe");
    server
}

// ---------------------------------------------------------------------------
// Test driver — spawn agent, run a scripted IPC exchange.
// ---------------------------------------------------------------------------

struct AgentHandle {
    child: tokio::process::Child,
    socket_path: PathBuf,
}

impl AgentHandle {
    async fn cleanup(mut self) {
        let _ = self.child.kill().await;
        let _ = self.child.wait().await;
        #[cfg(unix)]
        let _ = tokio::fs::remove_file(&self.socket_path).await;
        #[cfg(windows)]
        let _ = &self.socket_path;
    }
}

async fn spawn_agent(socket_path: &PathBuf, agent_id: &Uuid) -> tokio::process::Child {
    let bin = agent_binary_path();
    assert!(
        bin.exists(),
        "agent binary missing at {} — run `cargo build --bin elai_cc_agent` first",
        bin.display()
    );
    let mut cmd = tokio::process::Command::new(&bin);
    cmd.arg(socket_path).arg(agent_id.to_string());
    cmd.kill_on_drop(true);
    cmd.spawn().expect("spawn elai_cc_agent")
}

// Skip incidental heartbeats so the test driver only sees agent->orchestrator
// request frames it actually cares about.
async fn read_skipping_heartbeats<R>(reader: &mut R) -> Result<IpcMessageType, String>
where
    R: AsyncReadExt + Unpin,
{
    loop {
        match read_frame(reader).await? {
            IpcMessageType::AgentHeartbeat { .. } => continue,
            other => return Ok(other),
        }
    }
}

// ---------------------------------------------------------------------------
// Contract 1 — happy_path_two_iterations.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn happy_path_two_iterations() {
    let socket_id = Uuid::new_v4();
    let (listener, socket_path) = bind_endpoint(&socket_id).await;

    let child = spawn_agent(&socket_path, &socket_id).await;
    let handle = AgentHandle {
        child,
        socket_path: socket_path.clone(),
    };

    let outcome = tokio::time::timeout(Duration::from_secs(30), async {
        let mut stream = accept_endpoint(listener).await;

        // Step 1: send AgentRequest.
        let req = IpcMessageType::AgentRequest {
            task: "echo the number 42".to_owned(),
            dispatch_config: default_dispatch_config(),
        };
        write_frame(&mut stream, &req).await?;

        // Step 2: expect first InferRequest. Capture its prompt for later
        // assertion that the SECOND InferRequest contains the tool_id.
        let frame1 = read_skipping_heartbeats(&mut stream).await?;
        let prompt1 = match &frame1 {
            IpcMessageType::InferRequest { prompt, .. } => prompt.clone(),
            other => return Err(format!("expected InferRequest, got {other:?}")),
        };
        assert!(
            !prompt1.is_empty(),
            "first InferRequest must carry a non-empty prompt (got empty)"
        );

        // Step 3: reply with one TOOL_USE marker.
        let infer_resp_1 = IpcMessageType::InferResponse {
            text: "thinking...\nTOOL_USE: echo\n".to_owned(),
            tokens_used: 10,
        };
        write_frame(&mut stream, &infer_resp_1).await?;

        // Step 4: expect ToolRequest for `echo`.
        let frame2 = read_skipping_heartbeats(&mut stream).await?;
        let tool_id_seen = match &frame2 {
            IpcMessageType::ToolRequest { tool_id, .. } => tool_id.clone(),
            other => return Err(format!("expected ToolRequest, got {other:?}")),
        };
        assert_eq!(
            tool_id_seen.as_str(),
            "echo",
            "tool_id must round-trip from TOOL_USE marker"
        );

        // Step 5: reply ToolResult{output=b"42", success=true}.
        let tool_result = IpcMessageType::ToolResult {
            tool_id: tool_id_seen.clone(),
            output: b"42".to_vec(),
            success: true,
        };
        write_frame(&mut stream, &tool_result).await?;

        // Step 6: expect SECOND InferRequest. Its prompt MUST contain "echo"
        // (the tool_id from iteration 1) AND "42" (the tool output).
        let frame3 = read_skipping_heartbeats(&mut stream).await?;
        let prompt2 = match &frame3 {
            IpcMessageType::InferRequest { prompt, .. } => prompt.clone(),
            other => return Err(format!("expected second InferRequest, got {other:?}")),
        };
        assert!(
            prompt2.contains("echo"),
            "second InferRequest prompt must contain prior tool_id 'echo'; got: {prompt2}"
        );
        assert!(
            prompt2.contains("42"),
            "second InferRequest prompt must contain prior tool output '42'; got: {prompt2}"
        );

        // Step 7: reply InferResponse with NO TOOL_USE marker -> end_turn.
        let infer_resp_2 = IpcMessageType::InferResponse {
            text: "final answer: 42".to_owned(),
            tokens_used: 5,
        };
        write_frame(&mut stream, &infer_resp_2).await?;

        // Step 8: expect AgentResponse with non-empty content.
        let frame4 = read_skipping_heartbeats(&mut stream).await?;
        match frame4 {
            IpcMessageType::AgentResponse { content, .. } => {
                assert!(
                    !content.trim().is_empty(),
                    "AgentResponse content must be non-empty on end_turn"
                );
                Ok::<(), String>(())
            }
            other => Err(format!("expected AgentResponse, got {other:?}")),
        }
    })
    .await;

    handle.cleanup().await;
    match outcome {
        Ok(Ok(())) => {}
        Ok(Err(e)) => panic!("happy_path failed: {e}"),
        Err(_) => panic!("happy_path timed out after 30s"),
    }
}

// ---------------------------------------------------------------------------
// Contract 2 — max_iterations_cap. Drives 25 tool_use loops, expects the
// agent to emit IpcMessageType::AgentMaxIterationsReached on iteration 26
// instead of issuing another InferRequest.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn max_iterations_cap() {
    let socket_id = Uuid::new_v4();
    let (listener, socket_path) = bind_endpoint(&socket_id).await;

    let child = spawn_agent(&socket_path, &socket_id).await;
    let handle = AgentHandle {
        child,
        socket_path: socket_path.clone(),
    };

    let outcome = tokio::time::timeout(Duration::from_secs(60), async {
        let mut stream = accept_endpoint(listener).await;

        let req = IpcMessageType::AgentRequest {
            task: "infinite tool loop".to_owned(),
            dispatch_config: default_dispatch_config(),
        };
        write_frame(&mut stream, &req).await?;

        // 25 full InferRequest -> ToolRequest -> ToolResult round-trips.
        for i in 0..25 {
            // InferRequest #i
            match read_skipping_heartbeats(&mut stream).await? {
                IpcMessageType::InferRequest { .. } => {}
                other => return Err(format!("iter {i}: expected InferRequest, got {other:?}")),
            }
            // Reply with TOOL_USE marker.
            let infer_resp = IpcMessageType::InferResponse {
                text: format!("iter {i}\nTOOL_USE: echo\n"),
                tokens_used: 10,
            };
            write_frame(&mut stream, &infer_resp).await?;

            // ToolRequest
            let tool_id = match read_skipping_heartbeats(&mut stream).await? {
                IpcMessageType::ToolRequest { tool_id, .. } => tool_id,
                other => return Err(format!("iter {i}: expected ToolRequest, got {other:?}")),
            };
            // Reply ToolResult.
            write_frame(
                &mut stream,
                &IpcMessageType::ToolResult {
                    tool_id,
                    output: format!("ok{i}").into_bytes(),
                    success: true,
                },
            )
            .await?;
        }

        // After 25 round-trips, iteration counter == 25; the agent must NOT
        // send another InferRequest. Instead it must emit
        // AgentMaxIterationsReached and stop.
        match read_skipping_heartbeats(&mut stream).await? {
            IpcMessageType::AgentMaxIterationsReached {
                reason,
                iterations_completed,
            } => {
                assert!(
                    !reason.trim().is_empty(),
                    "AgentMaxIterationsReached.reason must be non-empty"
                );
                assert_eq!(
                    iterations_completed, 25,
                    "iterations_completed must equal AGENT_MAX_ITERATIONS"
                );
                Ok::<(), String>(())
            }
            IpcMessageType::AgentResponse { .. } => Err(
                "agent emitted AgentResponse instead of AgentMaxIterationsReached at iter cap"
                    .to_owned(),
            ),
            other => Err(format!(
                "expected AgentMaxIterationsReached at iter cap, got {other:?}"
            )),
        }
    })
    .await;

    handle.cleanup().await;
    match outcome {
        Ok(Ok(())) => {}
        Ok(Err(e)) => panic!("max_iterations_cap failed: {e}"),
        Err(_) => panic!("max_iterations_cap timed out after 60s"),
    }
}
