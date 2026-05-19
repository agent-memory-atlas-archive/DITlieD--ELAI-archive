// Phase B2b — listen-before-spawn contract test.
//
// Proves that AgentPoolBabyStepExecutor::execute() binds the IPC listener
// BEFORE spawning the agent process, and that the session completes with
// BabyStepOutcome::Passed when a stub agent connects and sends one InferRequest.
//
// TDD: this file was written BEFORE the executor changes landed; it compiled
// after the API contract in task_runner.rs + ipc_accept.rs was widened.
//
// Hermetic — no real agent binary is required. A tokio task plays the agent
// role: it receives the socket path via a oneshot channel that the executor
// sends AFTER bind_session_listener completes (proving listen-before-spawn
// ordering), connects to it, drives one InferRequest round-trip, and sends an
// AgentResponse frame so the session ends with AgentFinalState::Passed.
//
// Contract assertions:
//   1. BabyStepOutcome::Passed.
//   2. ScriptedAgentRpcServer::infer_calls == 1.
//   3. Socket path received via oneshot proves bind happened before spawn
//      (listen-before-spawn invariant: the executor sends the path only after
//      bind_session_listener returns Ok, before launch_agent_process fires).

use std::sync::Arc;
use std::sync::atomic::Ordering;

use elai_ipc_types::{IpcFrame, IpcMessageType, FRAME_HEADER_SIZE};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex as AsyncMutex;

use elai_core::SeededRng;
use elai_orchestrator::agent_pool::{AgentPoolConfig, InProcessAgentPool, ModelPool};
use elai_orchestrator::ipc_accept::test_support::ScriptedAgentRpcServer;
use elai_orchestrator::ipc_gateway::IpcGateway;
use elai_orchestrator::pipeline_tree_navigator::BabyStepOutcome;
use elai_orchestrator::sandbox::SelfModificationLocks;
use elai_orchestrator::task_runner::{
    AgentPoolBabyStepExecutor, BabyStepExecutionCtx, BabyStepExecutor,
};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

struct InfiniteVramPool;
impl ModelPool for InfiniteVramPool {
    fn available_vram(&self) -> u64 {
        u64::MAX
    }
}

fn make_pool() -> Arc<AsyncMutex<InProcessAgentPool<SeededRng>>> {
    Arc::new(AsyncMutex::new(InProcessAgentPool::new(
        AgentPoolConfig::default(),
        SeededRng::new(0xB2B_C0FF),
        Arc::new(InfiniteVramPool),
    )))
}

fn make_gateway(socket_dir: &std::path::Path) -> Arc<AsyncMutex<IpcGateway>> {
    Arc::new(AsyncMutex::new(IpcGateway::with_os_clock(
        socket_dir.to_path_buf(),
        SelfModificationLocks::new(),
    )))
}

/// Write one IPC message as a wire frame.
async fn write_frame<W: AsyncWriteExt + Unpin>(writer: &mut W, msg: &IpcMessageType) {
    let frame = IpcFrame::encode(msg).expect("encode");
    let wire = frame.to_wire_bytes().expect("to_wire_bytes");
    writer.write_all(&wire).await.expect("write_all");
    writer.flush().await.expect("flush");
}

/// Read one IPC frame.
async fn read_frame<R: AsyncReadExt + Unpin>(reader: &mut R) -> IpcMessageType {
    let mut header = [0u8; FRAME_HEADER_SIZE];
    reader.read_exact(&mut header).await.expect("read header");
    let len = u32::from_le_bytes([header[1], header[2], header[3], header[4]]) as usize;
    let mut payload = vec![0u8; len];
    if len > 0 {
        reader.read_exact(&mut payload).await.expect("read payload");
    }
    IpcFrame::decode(&payload).expect("decode frame")
}

// ---------------------------------------------------------------------------
// Contract 1: listen-before-spawn — BabyStepOutcome::Passed + infer_calls==1
//
// Platform note: the executor calls bind_session_listener which creates:
//   - Unix: a UnixListener (socket file exists after bind)
//   - Windows: a NamedPipeServer (pipe name, no filesystem entry)
// The stub agent receives the socket path via a oneshot channel that fires
// AFTER bind and BEFORE spawn — proving listen-before-spawn ordering.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_listen_before_spawn_passes_and_routes_infer_request() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_dir = tmp.path().join("ipc");
    std::fs::create_dir_all(&socket_dir).expect("create socket_dir");

    let pool = make_pool();
    let gateway = make_gateway(&socket_dir);
    let rpc_server = Arc::new(ScriptedAgentRpcServer::new("scripted_response_B2b"));

    // Oneshot channel: executor sends the bound socket path AFTER
    // bind_session_listener, BEFORE launch_agent_process. The stub agent
    // receives it and connects, proving listen-before-spawn ordering.
    let (path_tx, path_rx) = tokio::sync::oneshot::channel::<std::path::PathBuf>();

    // Stub agent task: waits for the socket path, connects, drives one
    // InferRequest/InferResponse round-trip, sends AgentResponse to end cleanly.
    let stub_agent = tokio::spawn(async move {
        // Receive socket path (proves bind happened before spawn).
        let socket_path = path_rx.await.expect("path_rx: sender dropped");

        // Give the executor a brief moment to reach accept_one.
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;

        // Connect to the already-bound socket.
        #[cfg(unix)]
        let mut stream = tokio::net::UnixStream::connect(&socket_path)
            .await
            .expect("stub: UnixStream::connect");

        #[cfg(windows)]
        let mut stream = {
            // ipc_accept::bind_session_listener maps the path stem to a Windows
            // named-pipe name: \\.\pipe\elai-<stem>.
            let stem = socket_path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "elai-ipc".to_string());
            let pipe_name = format!(r"\\.\pipe\elai-{stem}");
            // Retry in case the executor hasn't reached ConnectNamedPipe yet.
            let mut delay = std::time::Duration::from_millis(5);
            let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(8);
            loop {
                match tokio::net::windows::named_pipe::ClientOptions::new().open(&pipe_name) {
                    Ok(s) => break s,
                    Err(_) if tokio::time::Instant::now() < deadline => {
                        tokio::time::sleep(delay).await;
                        delay = (delay * 2).min(std::time::Duration::from_millis(100));
                    }
                    Err(e) => panic!("stub: NamedPipeClient::open failed: {e}"),
                }
            }
        };

        // patch-R4 — drain the orchestrator's initial AgentRequest frame
        // before exercising the B2b InferRequest round-trip. See R4 contract
        // test crates/contracts/tests/baby_step_initial_agent_request_contract.rs.
        let initial = read_frame(&mut stream).await;
        assert!(
            matches!(initial, IpcMessageType::AgentRequest { .. }),
            "expected initial AgentRequest from orchestrator, got: {initial:?}"
        );

        // Send InferRequest.
        write_frame(
            &mut stream,
            &IpcMessageType::InferRequest {
                prompt: "hello from stub agent".to_string(),
                max_tokens: 128,
            },
        )
        .await;

        // Read InferResponse back.
        let response = read_frame(&mut stream).await;
        match response {
            IpcMessageType::InferResponse { text, .. } => {
                assert_eq!(text, "scripted_response_B2b", "InferResponse text mismatch");
            }
            other => panic!("expected InferResponse, got: {other:?}"),
        }

        // Send AgentResponse to signal clean session completion (Passed).
        write_frame(
            &mut stream,
            &IpcMessageType::AgentResponse {
                content: "done".to_string(),
                artifacts: Vec::new(),
            },
        )
        .await;
    });

    // Build the executor. The fake_binary does not exist — launch_agent_process
    // will fail, but the stub agent (which connected via the notifier path)
    // has already driven the session to completion. The executor is designed to
    // honor the session outcome even when spawn fails (if accept_one succeeds).
    let fake_binary = tmp.path().join("fake-agent-b2b.exe");
    let executor = AgentPoolBabyStepExecutor::new(
        pool,
        fake_binary,
        socket_dir.clone(),
        gateway,
        rpc_server.clone() as Arc<dyn elai_orchestrator::AgentRpcServer + Send + Sync + 'static>,
    )
    .with_timeout_ms(10_000)
    .with_socket_path_notifier(path_tx);

    let ctx = BabyStepExecutionCtx {
        baby_step_id: 1,
        task_summary: "b2b-listen-before-spawn-contract".into(),
        escalation_message: None,
        mode_selection: None,
        session_credentials: std::collections::BTreeMap::new(),
    };

    let result = executor.execute(ctx).await;

    // Contract 1: session outcome is Passed.
    assert!(
        matches!(result.outcome, BabyStepOutcome::Passed),
        "expected Passed, got: {:?} (stderr: {})",
        result.outcome,
        result.stderr_tail,
    );

    // Contract 2: exactly one InferRequest routed to the scripted RPC server.
    assert_eq!(
        rpc_server.infer_calls.load(Ordering::Relaxed),
        1,
        "expected exactly 1 InferRequest routed to ScriptedAgentRpcServer"
    );

    stub_agent.await.expect("stub agent task panicked");
}
