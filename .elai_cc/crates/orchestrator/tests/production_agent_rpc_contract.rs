// Phase B2c — ProductionAgentRpcServer contract tests.
//
// TDD: these tests are written BEFORE the implementation exists so that
// a compilation failure confirms the module boundary is correct.
//
// Three contracts:
//   A. handle_infer_request routes to AnthropicClient, returns
//      InferOutcome::Response with the text from the stub server reply.
//   B. handle_tool_request for a known tool_id (file_read) returns
//      ToolOutcome::Result with deserializable bytes.
//   C. handle_tool_request for an unknown tool_id returns
//      ToolOutcome::Failed whose message starts with "unknown tool_id:".

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use elai_core::tool_dispatcher::{
    ClassifiedFailure, CommitOutcome, EditOutcome, EditRequest, FailureInput, FileView,
    GitCommitRequest, IncrementalOutcome, IncrementalRequest, ToolDispatchError, ToolDispatcher,
    ValidateRequest, ValidationTier, ValidationTierResult,
};
use elai_orchestrator::{
    AgentRpcServer, InferOutcome, ProductionAgentRpcServer, ToolOutcome,
};
use elai_providers::AnthropicClient;
use elai_tools::dispatcher::InProcessToolDispatcher;
use elai_tools::tiered_validation::TieredValidator;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

// ---------------------------------------------------------------------------
// No-op ToolDispatcher stub for tests that don't exercise tools.
// ---------------------------------------------------------------------------

struct NoOpDispatcher;

impl ToolDispatcher for NoOpDispatcher {
    fn dispatch_edit(
        &self,
        _req: EditRequest,
    ) -> Pin<Box<dyn Future<Output = Result<EditOutcome, ToolDispatchError>> + Send + '_>> {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
    }

    fn dispatch_validate(
        &self,
        _req: ValidateRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<ValidationTierResult>, ToolDispatchError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn dispatch_classify_failure(
        &self,
        _input: FailureInput,
    ) -> Pin<Box<dyn Future<Output = Option<ClassifiedFailure>> + Send + '_>> {
        Box::pin(async { None })
    }

    fn dispatch_git_commit(
        &self,
        _req: GitCommitRequest,
    ) -> Pin<
        Box<dyn Future<Output = Result<CommitOutcome, ToolDispatchError>> + Send + '_>,
    > {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
    }

    fn dispatch_file_read_outline(
        &self,
        _path: PathBuf,
        _line_threshold: usize,
    ) -> Pin<Box<dyn Future<Output = Result<FileView, ToolDispatchError>> + Send + '_>> {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
    }

    fn dispatch_incremental_compile(
        &self,
        _req: IncrementalRequest,
    ) -> Pin<
        Box<dyn Future<Output = Result<IncrementalOutcome, ToolDispatchError>> + Send + '_>,
    > {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
    }
}

// ---------------------------------------------------------------------------
// Minimal TieredValidator for constructing InProcessToolDispatcher in tests.
// Returns passed=true for every tier so file_read dispatches work without a
// real integration runner.
// ---------------------------------------------------------------------------

struct PassthroughValidator;

impl TieredValidator for PassthroughValidator {
    fn run_tier<'a>(
        &'a self,
        tier: ValidationTier,
        _req: &'a ValidateRequest,
    ) -> Pin<Box<dyn Future<Output = ValidationTierResult> + Send + 'a>> {
        Box::pin(async move {
            ValidationTierResult {
                tier,
                passed: true,
                elapsed_ms: 0,
                rules_fired: 0,
                stderr_snippet: None,
            }
        })
    }

    fn run_all<'a>(
        &'a self,
        _req: &'a ValidateRequest,
    ) -> Pin<Box<dyn Future<Output = Vec<ValidationTierResult>> + Send + 'a>> {
        Box::pin(async move { Vec::new() })
    }
}

// ---------------------------------------------------------------------------
// HTTP stub helpers
// ---------------------------------------------------------------------------

/// Minimal Anthropic /v1/messages JSON response carrying a single Text block.
fn canned_anthropic_response(text: &str) -> String {
    serde_json::json!({
        "id": "msg_test",
        "type": "message",
        "role": "assistant",
        "content": [{"type": "text", "text": text}],
        "model": "claude-test",
        "stop_reason": "end_turn",
        "usage": {
            "input_tokens": 10,
            "output_tokens": 5
        }
    })
    .to_string()
}

/// Spawn a one-shot TCP stub that reads one HTTP request then writes a canned
/// Anthropic response. Returns the base URL as `http://127.0.0.1:<port>`.
async fn spawn_anthropic_stub(response_text: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    let base_url = format!("http://{addr}");

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = vec![0u8; 8192];
            let _ = socket.read(&mut buf).await;
            let body = canned_anthropic_response(response_text);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body,
            );
            let _ = socket.write_all(response.as_bytes()).await;
            let _ = socket.shutdown().await;
        }
    });

    base_url
}

// ---------------------------------------------------------------------------
// Test A — handle_infer_request returns InferOutcome::Response(text)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_infer_request_returns_response_from_stub() {
    let stub_url = spawn_anthropic_stub("hello from stub").await;
    let client = AnthropicClient::new("test-key")
        .with_base_url(stub_url)
        .with_retry_policy(
            0,
            std::time::Duration::from_millis(0),
            std::time::Duration::from_millis(0),
        );
    let dispatcher: Arc<dyn elai_core::tool_dispatcher::ToolDispatcher> =
        Arc::new(NoOpDispatcher);
    let server = ProductionAgentRpcServer::new(client, dispatcher, "claude-test".into(), 100);

    let outcome = server.handle_infer_request("say hello".into(), 100).await;

    match outcome {
        InferOutcome::Response(text) => {
            assert_eq!(text, "hello from stub", "response text must match stub");
        }
        InferOutcome::Failed(reason) => {
            panic!("expected InferOutcome::Response, got Failed: {reason}");
        }
    }
}

// ---------------------------------------------------------------------------
// Test B — handle_tool_request for file_read returns ToolOutcome::Result
// Uses the REAL InProcessToolDispatcher with PassthroughValidator to exercise
// dispatch_file_read_outline against a real file in a tempdir.
// ---------------------------------------------------------------------------
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn b_file_read_tool_returns_result_bytes() {
    use std::io::Write as _;

    let dir = tempfile::TempDir::new().expect("tempdir");
    let file_path = dir.path().join("sample.txt");
    {
        let mut f = std::fs::File::create(&file_path).expect("create sample.txt");
        writeln!(f, "line one").expect("write");
        writeln!(f, "line two").expect("write");
    }

    let client = AnthropicClient::new("test-key").with_base_url("http://127.0.0.1:1".to_string());
    // Use the REAL InProcessToolDispatcher wired with PassthroughValidator.
    // This exercises the actual dispatch_file_read_outline code path rather than a stub.
    let validator: Arc<dyn TieredValidator> = Arc::new(PassthroughValidator);
    let dispatcher: Arc<dyn elai_core::tool_dispatcher::ToolDispatcher> =
        Arc::new(InProcessToolDispatcher::new(validator));
    let server = ProductionAgentRpcServer::new(client, dispatcher, "claude-test".into(), 100);

    let wire_req = elai_ipc_types::FileReadRequest {
        file_path: file_path.to_string_lossy().into_owned(),
        offset: None,
        limit: None,
    };
    let args = postcard::to_allocvec(&wire_req).expect("postcard encode");

    let outcome = server.handle_tool_request("file_read".into(), args).await;

    match outcome {
        ToolOutcome::Result(bytes) => {
            assert!(!bytes.is_empty(), "result bytes must not be empty");
            let parsed: serde_json::Value =
                serde_json::from_slice(&bytes).expect("result bytes must be valid JSON");
            // Verify it has the expected JSON shape.
            assert!(
                parsed.get("total_lines").is_some(),
                "result JSON must have total_lines key"
            );
        }
        ToolOutcome::Failed(reason) => {
            panic!("expected ToolOutcome::Result, got Failed: {reason}");
        }
    }
}

// ---------------------------------------------------------------------------
// Test C — unknown tool_id returns ToolOutcome::Failed("unknown tool_id: ...")
// ---------------------------------------------------------------------------

#[tokio::test]
async fn c_unknown_tool_id_returns_failed() {
    let client = AnthropicClient::new("test-key").with_base_url("http://127.0.0.1:1".to_string());
    let dispatcher: Arc<dyn elai_core::tool_dispatcher::ToolDispatcher> =
        Arc::new(NoOpDispatcher);
    let server = ProductionAgentRpcServer::new(client, dispatcher, "claude-test".into(), 100);

    let outcome = server
        .handle_tool_request("definitely_not_a_tool".into(), vec![])
        .await;

    match outcome {
        ToolOutcome::Failed(reason) => {
            assert!(
                reason.starts_with("unknown tool_id:"),
                "reason must start with 'unknown tool_id:', got: {reason}"
            );
        }
        ToolOutcome::Result(_) => {
            panic!("expected ToolOutcome::Failed for unknown tool_id");
        }
    }
}
