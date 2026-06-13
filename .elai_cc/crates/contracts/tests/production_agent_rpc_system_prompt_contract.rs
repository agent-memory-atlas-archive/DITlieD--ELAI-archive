// patch-R5 — ProductionAgentRpcServer system_prompt + tool_choice contract.
//
// Black-box contract: the system block + tool_choice published by
// ProductionAgentRpcServer on every MessageRequest control whether Sonnet
// emits ToolUse blocks at all. Without these, B3 cloud E2E observed Sonnet
// returning text-only responses and fw_rows=0 (see R4 handoff: agent runs one
// ReAct iteration, parses zero TOOL_USE markers, exits cleanly with no
// FileWrite SSB row).
//
// This test pins:
//   (1) `.with_system_prompt(p)` causes `MessageRequest.system` to be
//       `Some(vec![SystemBlock{kind:"text", text:p, ..}])` (one block, exact
//       text match).
//   (2) `.with_system_prompt(_)` does NOT mutate behaviour when absent:
//       a server built WITHOUT the builder call ships `system: None`.
//   (3) `MessageRequest.tool_choice == Some(ToolChoice::Auto)` for EVERY
//       infer call regardless of system_prompt presence. Auto is the right
//       default per the Anthropic docs (extended-thinking compatible; lets
//       the model emit natural-language reasoning interleaved with ToolUse
//       blocks; the directive system_prompt biases Sonnet into invoking
//       tools, not enforcement at the wire level).
//   (4) Adding `.with_system_prompt(_)` MUST NOT remove the 6-tool catalog
//       (regression guard that overlaps with the tool_catalog contract --
//       cheap, worth it; the catalog flowed through unchanged from R3.1).
//
// Failure modes this guards against:
//   - A future patch dropping the system_prompt field or wiring it under the
//     wrong builder name (silent regression -- Sonnet falls back to text-only).
//   - A future patch flipping tool_choice from Auto to None ("auto by
//     omission") -- some providers treat the two differently; explicit Auto
//     is the contract.
//   - A future patch leaking a hard-coded system_prompt into the default
//     `new()` ctor, breaking the 10+ existing call sites that don't opt in.
//
// Tested via the HTTP wire surface: spawn a TCP stub, run a real
// `handle_infer_request`, capture the request body, parse it as
// `MessageRequest`, then assert on `request.system` / `request.tool_choice`.

use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use elai_core::tool_dispatcher::{
    ClassifiedFailure, CommitOutcome, EditOutcome, EditRequest, FailureInput, FileView,
    GitCommitRequest, IncrementalOutcome, IncrementalRequest, ToolDispatchError, ToolDispatcher,
    ValidateRequest, ValidationTierResult,
};
use elai_orchestrator::ipc_accept::AgentRpcServer;
use elai_orchestrator::ProductionAgentRpcServer;
use elai_providers::{AnthropicClient, MessageRequest, ToolChoice};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

// ---------------------------------------------------------------------------
// NoOp dispatcher (none of the contracts here exercise the dispatcher).
// ---------------------------------------------------------------------------

struct NoOpDispatcher;

impl ToolDispatcher for NoOpDispatcher {
    fn dispatch_edit(
        &self,
        _req: EditRequest,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<EditOutcome, ToolDispatchError>> + Send + '_>>
    {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
    }

    fn dispatch_validate(
        &self,
        _req: ValidateRequest,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<Vec<ValidationTierResult>, ToolDispatchError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn dispatch_classify_failure(
        &self,
        _input: FailureInput,
    ) -> Pin<Box<dyn std::future::Future<Output = Option<ClassifiedFailure>> + Send + '_>> {
        Box::pin(async { None })
    }

    fn dispatch_git_commit(
        &self,
        _req: GitCommitRequest,
    ) -> Pin<
        Box<dyn std::future::Future<Output = Result<CommitOutcome, ToolDispatchError>> + Send + '_>,
    > {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
    }

    fn dispatch_file_read_outline(
        &self,
        _path: PathBuf,
        _line_threshold: usize,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<FileView, ToolDispatchError>> + Send + '_>>
    {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
    }

    fn dispatch_incremental_compile(
        &self,
        _req: IncrementalRequest,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<IncrementalOutcome, ToolDispatchError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
    }
}

// ---------------------------------------------------------------------------
// HTTP stub that captures the inbound request body. Mirrors the helper used by
// production_agent_rpc_tool_catalog_contract.rs (R3.1) -- same shape, same
// 64 KiB buffer behaviour, same canned 200 response.
// ---------------------------------------------------------------------------

async fn spawn_capturing_stub(
    response_body: String,
    captured_tx: oneshot::Sender<String>,
) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    let base_url = format!("http://{addr}");

    tokio::spawn(async move {
        let (mut socket, _) = match listener.accept().await {
            Ok(pair) => pair,
            Err(_) => return,
        };

        let mut accumulated: Vec<u8> = Vec::with_capacity(16 * 1024);
        let mut chunk = [0u8; 8 * 1024];
        let mut expected_total_len: Option<usize> = None;

        loop {
            let n = match socket.read(&mut chunk).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            };
            accumulated.extend_from_slice(&chunk[..n]);

            if expected_total_len.is_none() {
                if let Some(headers_end) = find_double_crlf(&accumulated) {
                    let headers = &accumulated[..headers_end];
                    let content_length = parse_content_length(headers).unwrap_or(0);
                    expected_total_len = Some(headers_end + 4 + content_length);
                }
            }

            if let Some(target) = expected_total_len {
                if accumulated.len() >= target {
                    break;
                }
            }
        }

        let captured_request = String::from_utf8_lossy(&accumulated).to_string();
        let _ = captured_tx.send(captured_request);

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response_body.len(),
            response_body,
        );
        let _ = socket.write_all(response.as_bytes()).await;
        let _ = socket.shutdown().await;
    });

    base_url
}

fn find_double_crlf(buf: &[u8]) -> Option<usize> {
    if buf.len() < 4 {
        return None;
    }
    for i in 0..=buf.len() - 4 {
        if &buf[i..i + 4] == b"\r\n\r\n" {
            return Some(i);
        }
    }
    None
}

// s-68264 test-cleanup: skip lines lacking `:` (e.g. the HTTP request-line)
// instead of returning early. See production_agent_rpc_tool_catalog_contract.rs
// for the full rationale; same bug, same fix, kept in sync intentionally.
fn parse_content_length(headers: &[u8]) -> Option<usize> {
    let text = std::str::from_utf8(headers).ok()?;
    for line in text.split("\r\n") {
        let mut parts = line.splitn(2, ':');
        let name = match parts.next() {
            Some(s) => s.trim(),
            None => continue,
        };
        let value = match parts.next() {
            Some(s) => s.trim(),
            None => continue,
        };
        if name.eq_ignore_ascii_case("content-length") {
            return value.parse::<usize>().ok();
        }
    }
    None
}

fn canned_anthropic_response(text: &str) -> String {
    serde_json::json!({
        "id": "msg_sysprompt",
        "type": "message",
        "role": "assistant",
        "content": [{"type": "text", "text": text}],
        "model": "claude-test",
        "stop_reason": "end_turn",
        "usage": {"input_tokens": 1, "output_tokens": 1}
    })
    .to_string()
}

fn extract_json_body(captured: &str) -> &str {
    captured
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("")
}

// ---------------------------------------------------------------------------
// CONTRACT 1: `.with_system_prompt(p)` ships `system: Some(vec![SystemBlock])`
// with the exact prompt text.
// ---------------------------------------------------------------------------

const TEST_SYSTEM_PROMPT: &str =
    "You are a tool-using coding agent. You MUST invoke tools; do not just explain.";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_with_system_prompt_publishes_system_block() {
    let (tx, rx) = oneshot::channel::<String>();
    let stub_url = spawn_capturing_stub(canned_anthropic_response("ack"), tx).await;
    let client = AnthropicClient::new("test-key")
        .with_base_url(stub_url)
        .with_retry_policy(0, Duration::from_millis(0), Duration::from_millis(0));
    let dispatcher: Arc<dyn ToolDispatcher> = Arc::new(NoOpDispatcher);
    let server = ProductionAgentRpcServer::new(client, dispatcher, "test-model".into(), 128)
        .with_system_prompt(TEST_SYSTEM_PROMPT);

    let _ = server.handle_infer_request("ping".into(), 128).await;

    let captured = rx.await.expect("stub must deliver captured request");
    let body = extract_json_body(&captured);
    let request: MessageRequest =
        serde_json::from_str(body).expect("captured body must parse as MessageRequest");

    let system = request
        .system
        .as_ref()
        .expect("R5: with_system_prompt must populate MessageRequest.system");
    assert_eq!(
        system.len(),
        1,
        "R5: system must contain exactly 1 SystemBlock; got {}",
        system.len(),
    );
    assert_eq!(
        system[0].kind, "text",
        "R5: SystemBlock kind must be \"text\"; got {:?}",
        system[0].kind,
    );
    assert_eq!(
        system[0].text, TEST_SYSTEM_PROMPT,
        "R5: SystemBlock.text must match the prompt passed to with_system_prompt",
    );
}

// ---------------------------------------------------------------------------
// CONTRACT 2: WITHOUT `.with_system_prompt(_)`, request.system stays None.
// Pins the opt-in behaviour so the 10+ existing 4-arg call sites keep working.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_without_system_prompt_keeps_system_none() {
    let (tx, rx) = oneshot::channel::<String>();
    let stub_url = spawn_capturing_stub(canned_anthropic_response("ack"), tx).await;
    let client = AnthropicClient::new("test-key")
        .with_base_url(stub_url)
        .with_retry_policy(0, Duration::from_millis(0), Duration::from_millis(0));
    let dispatcher: Arc<dyn ToolDispatcher> = Arc::new(NoOpDispatcher);
    // No .with_system_prompt() call -- 4-arg ctor form only.
    let server = ProductionAgentRpcServer::new(client, dispatcher, "test-model".into(), 128);

    let _ = server.handle_infer_request("ping".into(), 128).await;

    let captured = rx.await.expect("stub must deliver captured request");
    let body = extract_json_body(&captured);
    let request: MessageRequest =
        serde_json::from_str(body).expect("captured body must parse as MessageRequest");

    assert!(
        request.system.is_none(),
        "R5: without with_system_prompt, request.system must be None; got {:?}",
        request.system,
    );
}

// ---------------------------------------------------------------------------
// CONTRACT 3: tool_choice == Some(Auto) ALWAYS, with or without system_prompt.
// Two sub-cases prove the wire shape is stable across both server flavours.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_tool_choice_is_auto_with_system_prompt() {
    let (tx, rx) = oneshot::channel::<String>();
    let stub_url = spawn_capturing_stub(canned_anthropic_response("ack"), tx).await;
    let client = AnthropicClient::new("test-key")
        .with_base_url(stub_url)
        .with_retry_policy(0, Duration::from_millis(0), Duration::from_millis(0));
    let dispatcher: Arc<dyn ToolDispatcher> = Arc::new(NoOpDispatcher);
    let server = ProductionAgentRpcServer::new(client, dispatcher, "test-model".into(), 128)
        .with_system_prompt(TEST_SYSTEM_PROMPT);

    let _ = server.handle_infer_request("ping".into(), 128).await;

    let captured = rx.await.expect("stub must deliver captured request");
    let body = extract_json_body(&captured);
    let request: MessageRequest =
        serde_json::from_str(body).expect("captured body must parse as MessageRequest");

    assert_eq!(
        request.tool_choice,
        Some(ToolChoice::Auto),
        "R5: tool_choice must be Some(Auto) when system_prompt is set",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_tool_choice_is_auto_without_system_prompt() {
    let (tx, rx) = oneshot::channel::<String>();
    let stub_url = spawn_capturing_stub(canned_anthropic_response("ack"), tx).await;
    let client = AnthropicClient::new("test-key")
        .with_base_url(stub_url)
        .with_retry_policy(0, Duration::from_millis(0), Duration::from_millis(0));
    let dispatcher: Arc<dyn ToolDispatcher> = Arc::new(NoOpDispatcher);
    let server = ProductionAgentRpcServer::new(client, dispatcher, "test-model".into(), 128);

    let _ = server.handle_infer_request("ping".into(), 128).await;

    let captured = rx.await.expect("stub must deliver captured request");
    let body = extract_json_body(&captured);
    let request: MessageRequest =
        serde_json::from_str(body).expect("captured body must parse as MessageRequest");

    assert_eq!(
        request.tool_choice,
        Some(ToolChoice::Auto),
        "R5: tool_choice must be Some(Auto) even without system_prompt -- the 6-tool catalog \
         is always published, so tool_choice must always be explicit",
    );
}

// ---------------------------------------------------------------------------
// CONTRACT 4: enabling system_prompt does NOT shrink the 6-tool catalog.
// Cheap regression guard duplicating the tool_catalog contract from R3.1.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_system_prompt_does_not_drop_tool_catalog() {
    let (tx, rx) = oneshot::channel::<String>();
    let stub_url = spawn_capturing_stub(canned_anthropic_response("ack"), tx).await;
    let client = AnthropicClient::new("test-key")
        .with_base_url(stub_url)
        .with_retry_policy(0, Duration::from_millis(0), Duration::from_millis(0));
    let dispatcher: Arc<dyn ToolDispatcher> = Arc::new(NoOpDispatcher);
    let server = ProductionAgentRpcServer::new(client, dispatcher, "test-model".into(), 128)
        .with_system_prompt(TEST_SYSTEM_PROMPT);

    let _ = server.handle_infer_request("ping".into(), 128).await;

    let captured = rx.await.expect("stub must deliver captured request");
    let body = extract_json_body(&captured);
    let request: MessageRequest =
        serde_json::from_str(body).expect("captured body must parse as MessageRequest");

    let tools = request
        .tools
        .as_ref()
        .expect("R5: catalog must remain Some when system_prompt is set");
    assert_eq!(
        tools.len(),
        25,
        "R5 + C-24 + C-30: catalog must remain 25 tools (R3 6-tool baseline + \
         4 symbol_* + 4 dkg_* + 8 canvas_* + 2 git_* + index_artifacts + \
         list_artifacts + remove_artifacts + search_artifacts + \
         acknowledge_infeasible (I-8)) when \
         system_prompt is set; got {}",
        tools.len(),
    );
}
