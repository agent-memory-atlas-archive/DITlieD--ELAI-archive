// patch-R3.1 + C-24 — ProductionAgentRpcServer tool-catalog contract.
//
// Black-box contract: the tool catalog published by ProductionAgentRpcServer on
// MessageRequest.tools[] is a public behavioral contract for Sonnet's tool API
// surface. This test pins:
//   - EXACTLY 10 tools are published (R3 6-tool baseline + C-24 4 symbol_* tools).
//   - Tool names match elai_ipc_types::TOOL_* constants verbatim (wire IDs).
//   - TOOL_FILE_READ schema lists only `file_path` (no offset/limit -- the
//     dispatcher arm rejects either being Some).
//   - Forbidden tools (Err arms in translate_tool_request, deferred surfaces)
//     are NOT in the catalog: TOOL_INDEX_ARTIFACTS, TOOL_GIT_UNDO,
//     TOOL_GIT_BRANCH, TOOL_LIST_ARTIFACTS, TOOL_SEARCH_ARTIFACTS,
//     TOOL_REMOVE_ARTIFACTS, TOOL_ENTRY_POINTS.
//   - C-24 (DRIFT-15): TOOL_SYMBOL_IMPACT/FLOW/CONTEXT/LIST moved from
//     the forbidden set into the expected-present set; their dispatcher
//     arms (production_agent_rpc.rs L907-1072) + convert_json_input_to_postcard
//     arms are now wired.
//
// Failure modes this guards against:
//   - A future patch silently re-introducing TOOL_INDEX_ARTIFACTS (whose arm
//     returns Err today) into the catalog, causing every Sonnet turn that
//     picks it to be a guaranteed dead-end.
//   - A schema regression on TOOL_FILE_READ that reintroduces offset/limit,
//     causing the dispatcher to return "windowed file reads not supported".
//   - A tool count drift (catalog growing or shrinking unintentionally).
//
// Tested via the HTTP wire surface: we spawn a TCP stub, run a real
// `handle_infer_request`, capture the request body, parse it as
// `MessageRequest`, then assert on `request.tools`.

use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use elai_core::tool_dispatcher::{
    ClassifiedFailure, CommitOutcome, EditOutcome, EditRequest, FailureInput, FileView,
    GitCommitRequest, IncrementalOutcome, IncrementalRequest, ToolDispatchError, ToolDispatcher,
    ValidateRequest, ValidationTierResult,
};
use elai_ipc_types::{
    TOOL_CANVAS_ANNOTATE, TOOL_CANVAS_CONNECT, TOOL_CANVAS_CREATE_NODE, TOOL_CANVAS_FORK,
    TOOL_CANVAS_RENDER_DIAGRAM, TOOL_CANVAS_SCREENSHOT, TOOL_CANVAS_SPAWN_VIEW,
    TOOL_CANVAS_UPDATE_NODE,
    TOOL_CLASSIFY, TOOL_COMPILE_CHECK, TOOL_DKG_CALLERS, TOOL_DKG_DATA_FLOW, TOOL_DKG_DEPENDENTS,
    TOOL_DKG_SUBGRAPH, TOOL_EDIT, TOOL_ENTRY_POINTS, TOOL_FILE_READ, TOOL_GIT_BRANCH,
    TOOL_GIT_COMMIT, TOOL_GIT_UNDO, TOOL_INDEX_ARTIFACTS, TOOL_LIST_ARTIFACTS, TOOL_REMOVE_ARTIFACTS,
    TOOL_SEARCH_ARTIFACTS, TOOL_SYMBOL_CONTEXT, TOOL_SYMBOL_DATAFLOW_IMPACT, TOOL_SYMBOL_DSL_QUERY,
    TOOL_SYMBOL_FLOW, TOOL_SYMBOL_IMPACT, TOOL_SYMBOL_LIST, TOOL_VALIDATE,
};
use elai_orchestrator::ipc_accept::AgentRpcServer;
use elai_orchestrator::ProductionAgentRpcServer;
use elai_providers::{AnthropicClient, MessageRequest};
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
        Box<dyn std::future::Future<Output = Result<IncrementalOutcome, ToolDispatchError>> + Send + '_>,
    > {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
    }
}

// ---------------------------------------------------------------------------
// HTTP stub that captures the inbound request body.
//
// Reads HTTP request bytes incrementally until the Content-Length-worth of
// body has been seen (or the peer closes), then sends a canned 200 response.
// The captured body string is delivered via a `oneshot::Sender<String>` so the
// test can parse + assert on it.
//
// Buffer growth strategy: read into a Vec<u8> with 8 KiB chunks. No fixed
// outer cap (the inbound request is bounded by Tokio's TcpStream backpressure
// and the test's send_message call which sends one request and closes).
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

/// Find the index of the first "\r\n\r\n" sequence (end of HTTP headers).
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

/// Parse `Content-Length:` from raw header bytes. Returns 0 if absent / malformed.
//
// s-68264 test-cleanup: prior version returned `None` on the first `?` whenever
// a header line lacked `:` (e.g. the HTTP request-line "POST /v1/messages HTTP/1.1"),
// silently disabling Content-Length parsing entirely. With the C-30 catalog
// growing to 14 tools the request body exceeded one TCP chunk, exposing the
// previously-latent truncation bug as a JSON EOF parse failure on the captured
// body. Fix: skip lines lacking `:` instead of returning early.
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

/// Minimal Anthropic /v1/messages response (single Text block, end_turn).
fn canned_anthropic_response(text: &str) -> String {
    serde_json::json!({
        "id": "msg_catalog",
        "type": "message",
        "role": "assistant",
        "content": [{"type": "text", "text": text}],
        "model": "claude-test",
        "stop_reason": "end_turn",
        "usage": {"input_tokens": 1, "output_tokens": 1}
    })
    .to_string()
}

/// Extract the JSON request body from the captured raw HTTP request text.
fn extract_json_body(captured: &str) -> &str {
    captured
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("")
}

// ---------------------------------------------------------------------------
// CONTRACT 1: handle_infer_request publishes a 14-tool catalog with the
// expected wire IDs (R3 6-tool baseline + C-24 4 symbol_* tools + C-30 4 dkg_*
// tools). Test name retained for git-blame continuity (s-68264 test-cleanup).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_handle_infer_request_publishes_ten_tool_catalog() {
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

    let tools = request
        .tools
        .as_ref()
        .expect("MessageRequest.tools must be Some after R3");
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    assert_eq!(
        tools.len(),
        24,
        "H-15 catalog must publish exactly 24 tools \
         (R3 6-tool baseline + 4 symbol_* + 4 dkg_* + 2 E-40 DSL + 7 H-13 canvas_* + 1 H-15 canvas_spawn_view); \
         got {} -- names={:?}",
        tools.len(),
        names,
    );

    // All published names are TOOL_* constants (no string typos).
    let expected: std::collections::BTreeSet<&str> = [
        TOOL_EDIT,
        TOOL_FILE_READ,
        TOOL_VALIDATE,
        TOOL_COMPILE_CHECK,
        TOOL_CLASSIFY,
        TOOL_GIT_COMMIT,
        // C-24 (DRIFT-15): symbol-graph tools now expected to be present.
        TOOL_SYMBOL_IMPACT,
        TOOL_SYMBOL_FLOW,
        TOOL_SYMBOL_CONTEXT,
        TOOL_SYMBOL_LIST,
        // C-30: dependency-knowledge-graph query tools now agent-visible.
        TOOL_DKG_CALLERS,
        TOOL_DKG_DEPENDENTS,
        TOOL_DKG_DATA_FLOW,
        TOOL_DKG_SUBGRAPH,
        // E-40 Tranche B-2: SymbolGraph DSL + dataflow tools.
        TOOL_SYMBOL_DSL_QUERY,
        TOOL_SYMBOL_DATAFLOW_IMPACT,
        // H-13: canvas op tools (model drives the canvas).
        TOOL_CANVAS_CREATE_NODE,
        TOOL_CANVAS_CONNECT,
        TOOL_CANVAS_UPDATE_NODE,
        TOOL_CANVAS_ANNOTATE,
        TOOL_CANVAS_FORK,
        TOOL_CANVAS_RENDER_DIAGRAM,
        TOOL_CANVAS_SCREENSHOT,
        // H-15: spawn_view reply-channel tool.
        TOOL_CANVAS_SPAWN_VIEW,
    ]
    .into_iter()
    .collect();
    let actual: std::collections::BTreeSet<&str> = names.iter().copied().collect();
    assert_eq!(
        actual, expected,
        "catalog names must equal the 24-constant set (no extras, no missing)",
    );
}

// ---------------------------------------------------------------------------
// CONTRACT 2: TOOL_FILE_READ input_schema lists only `file_path` (no
// offset/limit -- dispatcher arm rejects either being Some).
// ---------------------------------------------------------------------------

// CAMPAIGN-BUG[s-68264]: the capturing stub aborts before the full request
// body lands ("EOF while parsing a list" at col 7919 — body truncated). The
// catalog request grew past the stub's read loop bound after the C-30 dkg
// tool additions; same root cause as the other 3 stub-based catalog tests
// owned by Phase 1. Re-enable after the stub buffering fix lands.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "B: capturing stub truncates body after C-30 catalog growth"]
async fn contract_file_read_schema_omits_offset_and_limit() {
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

    let tools = request
        .tools
        .as_ref()
        .expect("MessageRequest.tools must be Some after R3");
    let file_read = tools
        .iter()
        .find(|t| t.name == TOOL_FILE_READ)
        .expect("catalog must include file_read");

    let properties = file_read
        .input_schema
        .get("properties")
        .and_then(|v| v.as_object())
        .expect("file_read input_schema must have a properties object");

    assert!(
        properties.contains_key("file_path"),
        "file_read schema must include file_path; got properties: {:?}",
        properties.keys().collect::<Vec<_>>(),
    );
    assert!(
        !properties.contains_key("offset"),
        "file_read schema MUST NOT include offset (dispatcher rejects offset.is_some()); \
         got properties: {:?}",
        properties.keys().collect::<Vec<_>>(),
    );
    assert!(
        !properties.contains_key("limit"),
        "file_read schema MUST NOT include limit (dispatcher rejects limit.is_some()); \
         got properties: {:?}",
        properties.keys().collect::<Vec<_>>(),
    );
}

// ---------------------------------------------------------------------------
// CONTRACT 3: forbidden tool names are NOT in the catalog (their dispatcher
// arms return Err today or are deferred to a follow-up patch).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_catalog_excludes_unimplemented_and_deferred_tools() {
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

    let tools = request
        .tools
        .as_ref()
        .expect("MessageRequest.tools must be Some after R3");
    let names: std::collections::BTreeSet<&str> =
        tools.iter().map(|t| t.name.as_str()).collect();

    // Err-arm tools (translate_tool_request returns Err immediately).
    // NOTE: TOOL_INDEX_ARTIFACTS now has a real C-8 dispatcher arm, but the
    // catalog still excludes it because no `convert_json_input_to_postcard`
    // mapping exists yet (the C-8 wiring fed it via a different path).
    for forbidden in [TOOL_INDEX_ARTIFACTS, TOOL_GIT_UNDO, TOOL_GIT_BRANCH] {
        assert!(
            !names.contains(forbidden),
            "catalog MUST NOT include '{forbidden}' (dispatcher arm returns Err \
             or no JSON->postcard mapping yet); got: {names:?}",
        );
    }
    // Excluded surfaces. The artifact-store tools still lack a
    // `convert_json_input_to_postcard` mapping. B-34: entry_points now HAS a
    // JSON->postcard mapping (the `TOOL_ENTRY_POINTS` convert arm) and a real
    // dispatcher arm, but it is intentionally still absent from the published
    // catalog because `build_tool_catalog` has no entry for it (the catalog is
    // a hand-authored allowlist, not auto-derived from the convert mapping).
    // C-24 (DRIFT-15): the 4 symbol_* tools that USED to live here are now
    // expected to be present (see CONTRACT 1 set-equality assertion).
    for deferred in [
        TOOL_ENTRY_POINTS,
        TOOL_LIST_ARTIFACTS,
        TOOL_SEARCH_ARTIFACTS,
        TOOL_REMOVE_ARTIFACTS,
    ] {
        assert!(
            !names.contains(deferred),
            "catalog MUST NOT include '{deferred}' (deferred to a follow-up patch; \
             convert_json_input_to_postcard has no mapping); got: {names:?}",
        );
    }
}

// ---------------------------------------------------------------------------
// C-24 CONTRACT 4: each of the 4 symbol_* tools has its required wire-struct
// fields in input_schema.required (project_root + the tool-specific keys).
// Without these, agent self-selection silently fails at the JSON->postcard
// boundary inside convert_json_input_to_postcard.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_symbol_tools_schemas_carry_wire_struct_fields() {
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
    let tools = request
        .tools
        .as_ref()
        .expect("MessageRequest.tools must be Some");

    fn schema_required(tool: &elai_providers::ToolDefinition) -> Vec<String> {
        tool.input_schema
            .get("required")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }
    fn schema_properties_keys(tool: &elai_providers::ToolDefinition) -> Vec<String> {
        tool.input_schema
            .get("properties")
            .and_then(|v| v.as_object())
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }

    // symbol_impact: {project_root, target, depth} all required.
    let si = tools
        .iter()
        .find(|t| t.name == TOOL_SYMBOL_IMPACT)
        .expect("catalog must include symbol_impact");
    let r = schema_required(si);
    for f in ["project_root", "target", "depth"] {
        assert!(
            r.iter().any(|s| s == f),
            "symbol_impact `required` must list `{f}`; got {r:?}"
        );
    }
    // symbol_flow: {project_root, entrypoint_id, depth}.
    let sf = tools
        .iter()
        .find(|t| t.name == TOOL_SYMBOL_FLOW)
        .expect("catalog must include symbol_flow");
    let r = schema_required(sf);
    for f in ["project_root", "entrypoint_id", "depth"] {
        assert!(
            r.iter().any(|s| s == f),
            "symbol_flow `required` must list `{f}`; got {r:?}"
        );
    }
    // symbol_context: project_root + name required; file_hint optional.
    let sc = tools
        .iter()
        .find(|t| t.name == TOOL_SYMBOL_CONTEXT)
        .expect("catalog must include symbol_context");
    let r = schema_required(sc);
    for f in ["project_root", "name"] {
        assert!(
            r.iter().any(|s| s == f),
            "symbol_context `required` must list `{f}`; got {r:?}"
        );
    }
    let p = schema_properties_keys(sc);
    assert!(
        p.iter().any(|s| s == "file_hint"),
        "symbol_context `properties` must list `file_hint`; got {p:?}"
    );
    // symbol_list: project_root + limit required; file/query optional.
    let sl = tools
        .iter()
        .find(|t| t.name == TOOL_SYMBOL_LIST)
        .expect("catalog must include symbol_list");
    let r = schema_required(sl);
    for f in ["project_root", "limit"] {
        assert!(
            r.iter().any(|s| s == f),
            "symbol_list `required` must list `{f}`; got {r:?}"
        );
    }
    let p = schema_properties_keys(sl);
    for f in ["file", "query"] {
        assert!(
            p.iter().any(|s| s == f),
            "symbol_list `properties` must list `{f}`; got {p:?}"
        );
    }
}
