// patch-R4 — AgentPoolBabyStepExecutor initial AgentRequest frame contract.
//
// Proves that AgentPoolBabyStepExecutor::execute() writes an
// IpcMessageType::AgentRequest frame to the agent stream IMMEDIATELY after
// gateway-register and BEFORE entering run_agent_session. Without this frame
// the agent's run_message_loop blocks on read_frame() until the orchestrator's
// outer timeout closes the pipe — the symptom captured in
// workflow/handoff/CLOUD_E2E_PROGRESS_2026-05-17_R3_R3_1.md (fw_rows=0,
// "baby-step timed out at 120s").
//
// Canonical pattern mirrored: feasibility_probe.rs:718-733 (AgentRequest
// write site) + :524-547 (dispatch_config helper) + ipc_accept::write_frame_to_stream.
//
// TDD: this file was authored FIRST in TESTING_FIRST state. On HEAD prior to
// R4 the stub's first read returns a timeout/error because no frame is ever
// sent — the test FAILS. After R4 lands, the frame arrives, the stub decodes
// AgentRequest, asserts the shape, and the test PASSES.
//
// Contract assertions (set-equality with R3 catalog enforced):
//   1. First frame on the wire is IpcMessageType::AgentRequest.
//   2. `task` string contains the BabyStepExecutionCtx::task_summary substring.
//   3. `dispatch_config.role == AgentRole::Executor`.
//   4. `dispatch_config.tool_filter.allowed_tools` (sorted) equals
//      [classify, compile_check, edit, file_read, git_commit,
//       symbol_context, symbol_flow, symbol_impact, symbol_list, validate]
//      — the 10 wire IDs from production_agent_rpc::build_tool_catalog()
//      (R3 6-tool baseline + C-24 4 symbol_* additions).
//   5. `dispatch_config.sandbox_config.seccomp_profile == SeccompProfile::IpcBaseline`.
//   6. `dispatch_config.grc_budget_max_calls == DEFAULT_AGENT_GRC_MAX_CALLS` (5).
//   7. `dispatch_config.grc_budget_max_chars >= 600` (sane lower bound, no magic
//      number assertion — the R4 named constant value lives in task_runner.rs).
//   8. After the frame is consumed, the stub sends AgentResponse and the
//      executor reports BabyStepOutcome::Passed.

use std::sync::Arc;

use elai_ipc_types::{
    AgentRole, IpcFrame, IpcMessageType, SeccompProfile, FRAME_HEADER_SIZE, TOOL_CLASSIFY,
    TOOL_COMPILE_CHECK, TOOL_EDIT, TOOL_FILE_READ, TOOL_GIT_COMMIT, TOOL_SYMBOL_CONTEXT,
    TOOL_SYMBOL_FLOW, TOOL_SYMBOL_IMPACT, TOOL_SYMBOL_LIST, TOOL_VALIDATE,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex as AsyncMutex;

use elai_core::SeededRng;
use elai_orchestrator::agent_pool::{AgentPoolConfig, InProcessAgentPool, ModelPool};
use elai_orchestrator::ipc_accept::test_support::ScriptedAgentRpcServer;
use elai_orchestrator::ipc_gateway::IpcGateway;
use elai_orchestrator::pipeline_tree_navigator::BabyStepOutcome;
use elai_orchestrator::sandbox::SelfModificationLocks;
use elai_orchestrator::task_runner::{
    AgentPoolBabyStepExecutor, BabyStepExecutionCtx, BabyStepExecutor, DEFAULT_AGENT_GRC_MAX_CALLS,
};

// ---------------------------------------------------------------------------
// Test helpers (mirror agent_pool_listen_before_spawn_contract.rs)
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
        SeededRng::new(0x_C0FF_EE_04),
        Arc::new(InfiniteVramPool),
    )))
}

fn make_gateway(socket_dir: &std::path::Path) -> Arc<AsyncMutex<IpcGateway>> {
    Arc::new(AsyncMutex::new(IpcGateway::with_os_clock(
        socket_dir.to_path_buf(),
        SelfModificationLocks::new(),
    )))
}

async fn write_frame<W: AsyncWriteExt + Unpin>(writer: &mut W, msg: &IpcMessageType) {
    let frame = IpcFrame::encode(msg).expect("encode");
    let wire = frame.to_wire_bytes().expect("to_wire_bytes");
    writer.write_all(&wire).await.expect("write_all");
    writer.flush().await.expect("flush");
}

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
// Contract: AgentRequest is the FIRST frame on the wire after gateway-register.
// ---------------------------------------------------------------------------

const TASK_SUMMARY_SENTINEL: &str = "patch-R4-initial-agent-request-contract";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_initial_agent_request_frame_is_written() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_dir = tmp.path().join("ipc");
    std::fs::create_dir_all(&socket_dir).expect("create socket_dir");

    let pool = make_pool();
    let gateway = make_gateway(&socket_dir);
    let rpc_server = Arc::new(ScriptedAgentRpcServer::new("r4-stub-infer-response"));

    let (path_tx, path_rx) = tokio::sync::oneshot::channel::<std::path::PathBuf>();

    // Stub agent: read the FIRST frame (must be AgentRequest), validate its
    // shape, then send AgentResponse to end the session cleanly.
    let stub_agent = tokio::spawn(async move {
        let socket_path = path_rx.await.expect("path_rx: sender dropped");
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;

        #[cfg(unix)]
        let mut stream = tokio::net::UnixStream::connect(&socket_path)
            .await
            .expect("stub: UnixStream::connect");

        #[cfg(windows)]
        let mut stream = {
            let stem = socket_path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "elai-ipc".to_string());
            let pipe_name = format!(r"\\.\pipe\elai-{stem}");
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

        // === Contract assertion (1): first frame is AgentRequest. ===
        let first = read_frame(&mut stream).await;
        let (task, dispatch_config) = match first {
            IpcMessageType::AgentRequest { task, dispatch_config } => (task, dispatch_config),
            other => panic!(
                "expected first frame to be AgentRequest, got: {other:?}\n\
                 R4 contract violated — orchestrator never wrote initial AgentRequest"
            ),
        };

        // === Contract assertion (2): task carries the sentinel summary. ===
        assert!(
            task.contains(TASK_SUMMARY_SENTINEL),
            "AgentRequest.task missing sentinel substring '{TASK_SUMMARY_SENTINEL}'; got: {task:?}"
        );

        // === Contract assertion (3): role is Executor. ===
        assert_eq!(
            dispatch_config.role,
            AgentRole::Executor,
            "dispatch_config.role must be Executor (baby-step default)"
        );

        // === Contract assertion (4): tool_filter set-equality with the
        //     C-24 catalog (R3 6-tool baseline + 4 symbol_* additions). ===
        let mut tools: Vec<String> = dispatch_config
            .tool_filter
            .allowed_tools
            .iter()
            .map(|t| t.0.clone())
            .collect();
        tools.sort();
        let mut expected: Vec<String> = vec![
            TOOL_EDIT.to_string(),
            TOOL_FILE_READ.to_string(),
            TOOL_VALIDATE.to_string(),
            TOOL_COMPILE_CHECK.to_string(),
            TOOL_CLASSIFY.to_string(),
            TOOL_GIT_COMMIT.to_string(),
            // C-24 (DRIFT-13): symbol-graph tools added in lockstep with
            // build_tool_catalog. baby_step_dispatch_config.allowed_tools
            // MUST match the published catalog or the agent's tool_filter
            // strips them out even though the catalog publishes them.
            TOOL_SYMBOL_IMPACT.to_string(),
            TOOL_SYMBOL_FLOW.to_string(),
            TOOL_SYMBOL_CONTEXT.to_string(),
            TOOL_SYMBOL_LIST.to_string(),
        ];
        expected.sort();
        assert_eq!(
            tools, expected,
            "C-24 ToolFilter.allowed_tools must equal the 10-tool catalog \
             (R3 6-tool baseline + 4 symbol_*)"
        );

        // === Contract assertion (5): seccomp_profile == IpcBaseline. ===
        assert_eq!(
            dispatch_config.sandbox_config.seccomp_profile,
            SeccompProfile::IpcBaseline,
            "sandbox_config.seccomp_profile must be IpcBaseline"
        );

        // === Contract assertion (6): grc_budget_max_calls == named constant. ===
        assert_eq!(
            dispatch_config.grc_budget_max_calls, DEFAULT_AGENT_GRC_MAX_CALLS,
            "grc_budget_max_calls must equal DEFAULT_AGENT_GRC_MAX_CALLS"
        );

        // === Contract assertion (7): grc_budget_max_chars sane lower bound. ===
        assert!(
            dispatch_config.grc_budget_max_chars >= 600,
            "grc_budget_max_chars must be >= 600 (got {})",
            dispatch_config.grc_budget_max_chars
        );

        // Drive one InferRequest round-trip so the scripted RPC server
        // observes a turn (mirrors B2b regression coverage).
        write_frame(
            &mut stream,
            &IpcMessageType::InferRequest {
                prompt: "hello after r4 frame".to_string(),
                max_tokens: 32,
            },
        )
        .await;
        let response = read_frame(&mut stream).await;
        match response {
            IpcMessageType::InferResponse { text, .. } => {
                assert_eq!(text, "r4-stub-infer-response");
            }
            other => panic!("expected InferResponse, got: {other:?}"),
        }

        // Send AgentResponse to signal clean session completion.
        write_frame(
            &mut stream,
            &IpcMessageType::AgentResponse {
                content: "r4-ok".to_string(),
                artifacts: Vec::new(),
            },
        )
        .await;
    });

    let fake_binary = tmp.path().join("fake-agent-r4.exe");
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
        task_summary: TASK_SUMMARY_SENTINEL.into(),
        escalation_message: None,
        mode_selection: None,
        session_credentials: std::collections::BTreeMap::new(),
    };

    let result = executor.execute(ctx).await;

    // === Contract assertion (8): session completes as Passed. ===
    assert!(
        matches!(result.outcome, BabyStepOutcome::Passed),
        "expected Passed, got: {:?} (stderr: {})",
        result.outcome,
        result.stderr_tail,
    );

    stub_agent.await.expect("stub agent task panicked");
}
