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
    ClassifiedFailure, CommitOutcome, EditMatchTier, EditOutcome, EditRequest, FailureInput,
    FileView, GitCommitRequest, IncrementalOutcome, IncrementalRequest, ToolDispatchError,
    ToolDispatcher, ValidateRequest, ValidationTier, ValidationTierResult,
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
// EditEchoDispatcher — returns a synthetic successful EditOutcome for
// dispatch_edit; every other method returns NotImplemented/empty. Used by
// test (e) so the assertion focuses on the R3 stash-pop + JSON->postcard
// conversion path, not on the real edit-tier matching logic of
// InProcessToolDispatcher (which would require a real Cargo project layout).
// ---------------------------------------------------------------------------

struct EditEchoDispatcher;

impl ToolDispatcher for EditEchoDispatcher {
    fn dispatch_edit(
        &self,
        req: EditRequest,
    ) -> Pin<Box<dyn Future<Output = Result<EditOutcome, ToolDispatchError>> + Send + '_>> {
        Box::pin(async move {
            let bytes_written: u32 = req.new_text.len().try_into().unwrap_or(u32::MAX);
            Ok(EditOutcome {
                tier: EditMatchTier::Exact,
                bytes_written,
                file_path: req.file_path,
                similarity_hint: None,
            })
        })
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

/// R3: Anthropic /v1/messages JSON response carrying one Text block and one
/// ToolUse block. Mirrors `canned_anthropic_response` shape; the tool_use
/// block uses the snake_case `"tool_use"` tag (matches the `OutputContentBlock`
/// `#[serde(tag="type", rename_all="snake_case")]` enum surface).
fn canned_anthropic_response_with_tool_use(
    text: &str,
    tool_use_id: &str,
    tool_use_name: &str,
    tool_use_input: serde_json::Value,
) -> String {
    serde_json::json!({
        "id": "msg_test",
        "type": "message",
        "role": "assistant",
        "content": [
            {"type": "text", "text": text},
            {
                "type": "tool_use",
                "id": tool_use_id,
                "name": tool_use_name,
                "input": tool_use_input
            }
        ],
        "model": "claude-test",
        "stop_reason": "tool_use",
        "usage": {
            "input_tokens": 10,
            "output_tokens": 5
        }
    })
    .to_string()
}

/// R3: variant of `spawn_anthropic_stub` that accepts an owned response body
/// so callers can pass a dynamically-constructed JSON string (needed when the
/// response includes a tempdir-derived file_path that cannot be `'static`).
async fn spawn_anthropic_stub_owned(response_body: String) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    let base_url = format!("http://{addr}");

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            // 64 KiB — comfortably above the request body that R3 produces
            // (prompt + 6-tool catalog ~= 3 KiB) with headroom for follow-up
            // patches that may extend the catalog. A single `read` is still
            // not guaranteed to deliver the whole HTTP request in one call,
            // but the orchestrator's send_message issues one POST and we
            // respond once the response headers are written; partial reads
            // here would only matter if we needed to *parse* the request.
            let mut buf = vec![0u8; 64 * 1024];
            let _ = socket.read(&mut buf).await;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body,
            );
            let _ = socket.write_all(response.as_bytes()).await;
            let _ = socket.shutdown().await;
        }
    });

    base_url
}

/// Spawn a one-shot TCP stub that reads one HTTP request then writes a canned
/// Anthropic response. Returns the base URL as `http://127.0.0.1:<port>`.
async fn spawn_anthropic_stub(response_text: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    let base_url = format!("http://{addr}");

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            // 64 KiB -- see spawn_anthropic_stub_owned for rationale.
            let mut buf = vec![0u8; 64 * 1024];
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

// ---------------------------------------------------------------------------
// Test D — handle_infer_request emits "TOOL_USE: <name>" marker and stashes
// the ToolUse.input into pending_tool_args (R3).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn d_infer_request_emits_tool_use_marker_from_canned_sonnet() {
    let body = canned_anthropic_response_with_tool_use(
        "working on it",
        "toolu_abc123",
        "edit",
        serde_json::json!({
            "file_path": "src/main.rs",
            "old_text": "todo",
            "new_text": "done"
        }),
    );
    let stub_url = spawn_anthropic_stub_owned(body).await;
    let client = AnthropicClient::new("test-key")
        .with_base_url(stub_url)
        .with_retry_policy(
            0,
            std::time::Duration::from_millis(0),
            std::time::Duration::from_millis(0),
        );
    let dispatcher: Arc<dyn elai_core::tool_dispatcher::ToolDispatcher> =
        Arc::new(NoOpDispatcher);
    let server = ProductionAgentRpcServer::new(client, dispatcher, "test-model".into(), 1024);

    let outcome = server.handle_infer_request("prompt".into(), 1024).await;

    match outcome {
        InferOutcome::Response(text) => {
            assert!(
                text.contains("working on it"),
                "response text must include the Text block content, got: {text:?}"
            );
            assert!(
                text.lines().any(|l| l.trim_start() == "TOOL_USE: edit"),
                "response text must include a 'TOOL_USE: edit' marker line, got: {text:?}"
            );
        }
        InferOutcome::Failed(reason) => {
            panic!("expected InferOutcome::Response, got Failed: {reason}");
        }
    }

    let guard = server.pending_tool_args.lock().await;
    assert_eq!(
        guard.get("edit").map(|q| q.len()),
        Some(1),
        "ToolUse.input must be stashed under name 'edit' with exactly one entry"
    );
}

// ---------------------------------------------------------------------------
// Test E — handle_tool_request pops stashed args when arguments.is_empty(),
// converts JSON->postcard, and dispatches the edit arm to a successful Result.
// Stash must be drained to len=0 after dispatch (R3).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn e_handle_tool_request_pops_stashed_args_and_dispatches_edit() {
    let body = canned_anthropic_response_with_tool_use(
        "applying edit",
        "toolu_edit1",
        "edit",
        serde_json::json!({
            "file_path": "src/main.rs",
            "old_text": "todo",
            "new_text": "done"
        }),
    );
    let stub_url = spawn_anthropic_stub_owned(body).await;
    let client = AnthropicClient::new("test-key")
        .with_base_url(stub_url)
        .with_retry_policy(
            0,
            std::time::Duration::from_millis(0),
            std::time::Duration::from_millis(0),
        );
    let dispatcher: Arc<dyn elai_core::tool_dispatcher::ToolDispatcher> =
        Arc::new(EditEchoDispatcher);
    let server = ProductionAgentRpcServer::new(client, dispatcher, "test-model".into(), 1024);

    // Populate the stash by running the infer round-trip first.
    let _ = server.handle_infer_request("prompt".into(), 1024).await;

    // Agent_loop sends ToolRequest with empty arguments (agent_loop.rs:170);
    // the R3 path must pop the stash, convert JSON -> postcard, dispatch.
    let outcome = server.handle_tool_request("edit".into(), Vec::new()).await;

    match outcome {
        ToolOutcome::Result(bytes) => {
            let parsed: serde_json::Value =
                serde_json::from_slice(&bytes).expect("result bytes must be valid JSON");
            assert!(
                parsed.get("file_path").is_some(),
                "edit result JSON must carry file_path key, got: {parsed}"
            );
        }
        ToolOutcome::Failed(reason) => {
            panic!("expected ToolOutcome::Result, got Failed: {reason}");
        }
    }

    let guard = server.pending_tool_args.lock().await;
    assert_eq!(
        guard.get("edit").map(|q| q.len()),
        Some(0),
        "stashed 'edit' entry must be popped to len=0 after dispatch"
    );
}
