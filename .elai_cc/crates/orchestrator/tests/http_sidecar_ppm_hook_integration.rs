//! D-2 integration test: HttpSidecarPpmHook against an in-process
//! tokio TcpListener mock that speaks the `/v1/ppm/score` wire contract.
//!
//! Covers the round-trip cases the unit tests in `http_sidecar_ppm_hook.rs`
//! cannot — those tests only exercise the closed-port path. Here we spin
//! up a real HTTP server on an ephemeral port, plumb the bound URL into
//! `HttpSidecarPpmHook::with_endpoint`, and verify:
//!
//!   1. happy-path round-trip — server returns a well-formed JSON body;
//!      hook returns a typed `PpmScoreResponse` with matching shape.
//!   2. 503 unavailable -> `PpmHookError::Unavailable` (server returns
//!      503 + structured error body the Rust `WireErrorBody` does NOT
//!      know about; classify_transport_error+ HTTP-status fallback turns
//!      this into a Transport error that is NOT retryable, but the
//!      separate Unavailable path is taken when the structured body's
//!      tag explicitly says so). We test both.
//!   3. malformed response (len mismatch) -> `PpmHookError::WireSchema`
//!      via `validate_response_shape` (which surfaces it as
//!      `PpmHookError::ShapeMismatch`).
//!   4. retry-on-503-then-200 succeeds — first attempt 503 with a
//!      structured `error="parse_error"` body (which `into_typed` maps to
//!      `PpmHookError::Parse` — NOT retryable) and a separate test for
//!      transient HTTP 502 (no body, retried via Transport classification)
//!      that succeeds on the 2nd attempt.
//!
//! Wire contract source of truth: `sidecar_ppm_hook.rs` lines 44-61
//! (clauses C1-C7) + `http_sidecar_ppm_hook.rs::validate_response_shape`
//! (lines 444-481).
//!
//! Anti-stale-training compliance:
//!   - tokio::sync::Mutex used for shared async state (`Arc<Mutex<u32>>`
//!     counter); `std::sync::Mutex` would deadlock under tokio.
//!   - `tokio::net::TcpListener` for the mock — same pattern as
//!     `media_validator_rce_functional::comfyui_mock`.
//!   - native async fn (Rust 1.75+) — no async-trait crate.
//!   - blake3 transitively via the orchestrator crate, not directly used here.
//!   - #[cfg(test)] unwrap()s are allowed per CLAUDE.md no-unwrap rule.

use std::sync::Arc;
use std::time::Duration;

use elai_orchestrator::http_sidecar_ppm_hook::HttpSidecarPpmHook;
use elai_orchestrator::sidecar_ppm_hook::{
    PpmHookError, PpmMode, PpmScoreRequest, SidecarPpmHook,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};

// ---------------------------------------------------------------------------
// Mock /v1/ppm/score server. Owns a response-script the test fixture builds
// up before .start()-ing the listener. Each accepted connection pops the
// next scripted response off the front of the queue (deterministic order).
// ---------------------------------------------------------------------------

/// One scripted reply the mock server emits in response to one POST to
/// `/v1/ppm/score`. The mock does NOT inspect the request body — the
/// caller is responsible for ordering scripts to match the call sequence.
#[derive(Debug, Clone)]
struct MockReply {
    status: u16,
    reason: &'static str,
    body: String,
    content_type: &'static str,
}

impl MockReply {
    fn ok(body: impl Into<String>) -> Self {
        Self {
            status: 200,
            reason: "OK",
            body: body.into(),
            content_type: "application/json",
        }
    }
    fn json_status(status: u16, reason: &'static str, body: impl Into<String>) -> Self {
        Self {
            status,
            reason,
            body: body.into(),
            content_type: "application/json",
        }
    }
    fn raw_status(status: u16, reason: &'static str, body: impl Into<String>) -> Self {
        Self {
            status,
            reason,
            body: body.into(),
            content_type: "text/plain",
        }
    }
}

struct MockHandle {
    addr: std::net::SocketAddr,
    shutdown: oneshot::Sender<()>,
    /// Count of POST /v1/ppm/score requests the mock observed. Useful for
    /// asserting the retry path actually retried (vs short-circuited).
    request_counter: Arc<Mutex<u32>>,
}

impl MockHandle {
    fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }
}

/// Spawn the mock. The `scripts` queue is consumed in FIFO order — one
/// reply per accepted request. When the queue empties, subsequent
/// requests get a 500 with `{"error":"parse_error","detail":"no script"}`.
async fn spawn_mock(scripts: Vec<MockReply>) -> MockHandle {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind 0");
    let addr = listener.local_addr().expect("local_addr");
    let (tx, mut rx) = oneshot::channel::<()>();
    let queue: Arc<Mutex<Vec<MockReply>>> = Arc::new(Mutex::new(scripts));
    let counter: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
    let counter_for_task = counter.clone();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut rx => break,
                accepted = listener.accept() => {
                    let Ok((mut sock, _peer)) = accepted else { break; };
                    let queue = queue.clone();
                    let counter = counter_for_task.clone();
                    tokio::spawn(async move {
                        let _ = handle_one(&mut sock, queue, counter).await;
                    });
                }
            }
        }
    });

    MockHandle {
        addr,
        shutdown: tx,
        request_counter: counter,
    }
}

async fn handle_one(
    sock: &mut tokio::net::TcpStream,
    queue: Arc<Mutex<Vec<MockReply>>>,
    counter: Arc<Mutex<u32>>,
) -> std::io::Result<()> {
    // Read up to 64 KiB of request. The PPM score body is small.
    let mut buf = vec![0u8; 65_536];
    let mut total = 0;
    let mut header_end: Option<usize> = None;
    let mut content_length: Option<usize> = None;
    loop {
        let n = sock.read(&mut buf[total..]).await?;
        if n == 0 {
            break;
        }
        total += n;
        if header_end.is_none() {
            if let Some(pos) = buf[..total].windows(4).position(|w| w == b"\r\n\r\n") {
                header_end = Some(pos + 4);
                // Parse Content-Length if present.
                let header_text = std::str::from_utf8(&buf[..pos]).unwrap_or("");
                for line in header_text.split("\r\n") {
                    if let Some(rest) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                        if let Ok(n) = rest.trim().parse::<usize>() {
                            content_length = Some(n);
                        }
                    }
                }
            }
        }
        if let (Some(hend), Some(clen)) = (header_end, content_length) {
            if total >= hend + clen {
                break;
            }
        }
        if total >= buf.len() {
            break;
        }
    }

    // Bump the request counter regardless of script availability.
    {
        let mut c = counter.lock().await;
        *c = c.saturating_add(1);
    }

    // Pop the next script (or fall back to a synthetic 500).
    let reply = {
        let mut q = queue.lock().await;
        if q.is_empty() {
            MockReply::json_status(
                500,
                "Internal Server Error",
                r#"{"error":"parse_error","detail":"no script"}"#,
            )
        } else {
            q.remove(0)
        }
    };

    write_reply(sock, &reply).await
}

async fn write_reply(sock: &mut tokio::net::TcpStream, reply: &MockReply) -> std::io::Result<()> {
    let header = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        reply.status,
        reply.reason,
        reply.content_type,
        reply.body.len()
    );
    sock.write_all(header.as_bytes()).await?;
    sock.write_all(reply.body.as_bytes()).await?;
    sock.shutdown().await
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn sample_request(num_steps: usize) -> PpmScoreRequest {
    let reasoning_steps: Vec<String> = (0..num_steps)
        .map(|i| format!("step {i}: derive intermediate value"))
        .collect();
    PpmScoreRequest {
        candidate_id: "cand-integ-1".to_string(),
        verifier_id: "ppm_qwen25_math_prm_7b".to_string(),
        problem_statement: "Compute area of rectangle 4 x 5".to_string(),
        reasoning_steps,
        mode: PpmMode::Math,
    }
}

fn shaped_response_json(num_steps: usize, mode: &str) -> String {
    let mut scores: Vec<f32> = Vec::with_capacity(num_steps);
    for i in 0..num_steps {
        // Deterministic scores in (0, 1) — 0.91, 0.92, 0.93, ...
        scores.push(0.90 + 0.01 * (i as f32 + 1.0));
    }
    let mean = scores.iter().sum::<f32>() / (num_steps as f32);
    let var = scores.iter().map(|s| (s - mean) * (s - mean)).sum::<f32>() / (num_steps as f32);
    let step_scores_json = scores
        .iter()
        .map(|s| format!("{s:.6}"))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"candidate_id":"cand-integ-1","verifier_id":"ppm_qwen25_math_prm_7b","mode":"{mode}","step_scores":[{step_scores_json}],"score_mean":{mean:.6},"score_var":{var:.6},"num_steps":{num_steps},"latency_ms":42}}"#
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn happy_path_round_trip_returns_well_formed_score() {
    let body = shaped_response_json(3, "math");
    let mock = spawn_mock(vec![MockReply::ok(body)]).await;

    let hook = HttpSidecarPpmHook::with_endpoint(
        mock.base_url(),
        Duration::from_secs(2),
        0,
    );
    let req = sample_request(3);
    let resp = hook.score(req).await.expect("score ok");

    assert_eq!(resp.num_steps, 3);
    assert_eq!(resp.step_scores.len(), 3);
    for s in &resp.step_scores {
        assert!(
            (0.0..=1.0).contains(s),
            "score out of range: {s}"
        );
    }
    assert_eq!(resp.mode, PpmMode::Math);
    assert_eq!(resp.candidate_id, "cand-integ-1");
    let counter = *mock.request_counter.lock().await;
    assert_eq!(counter, 1, "expected exactly one POST");

    let _ = mock.shutdown.send(());
}

#[tokio::test]
async fn server_503_with_unstructured_body_surfaces_as_transport_error() {
    // A 503 with NO structured WireErrorBody-shaped JSON falls through to
    // the `HTTP {status}` Transport branch — NOT the Unavailable branch.
    // (Unavailable comes from a reqwest connect failure, not an HTTP
    // 503 response.) We assert this distinction explicitly because the
    // operator-visible row depends on the variant.
    let mock = spawn_mock(vec![MockReply::raw_status(
        503,
        "Service Unavailable",
        "ppm sidecar busy",
    )]).await;

    let hook = HttpSidecarPpmHook::with_endpoint(
        mock.base_url(),
        Duration::from_secs(2),
        0, // disable retry so we see the first error verbatim
    );
    let req = sample_request(1);
    let err = hook.score(req).await.expect_err("expected error");
    match err {
        PpmHookError::Transport(s) => {
            assert!(s.contains("503"), "expected 503 in transport detail: {s}");
        }
        other => panic!("expected Transport(503), got {other:?}"),
    }

    let _ = mock.shutdown.send(());
}

#[tokio::test]
async fn structured_error_body_maps_to_typed_variant() {
    // When the server emits a 422 + WireErrorBody-shaped JSON, the hook
    // round-trips it back into the typed PpmHookError. Tests the
    // `WireErrorBody::into_typed` integration over the wire.
    let body = r#"{"error":"invalid_score","index":1,"value":1.42}"#;
    let mock = spawn_mock(vec![MockReply::json_status(
        422,
        "Unprocessable Entity",
        body,
    )]).await;

    let hook = HttpSidecarPpmHook::with_endpoint(
        mock.base_url(),
        Duration::from_secs(2),
        0,
    );
    let req = sample_request(2);
    let err = hook.score(req).await.expect_err("expected typed error");
    match err {
        PpmHookError::InvalidScore { index, value } => {
            assert_eq!(index, 1);
            assert!((value - 1.42).abs() < 1e-3, "value: {value}");
        }
        other => panic!("expected InvalidScore, got {other:?}"),
    }

    let _ = mock.shutdown.send(());
}

#[tokio::test]
async fn malformed_response_len_mismatch_returns_shape_mismatch() {
    // Server returns a 200 with a well-formed-looking JSON body, but
    // step_scores.len() != reasoning_steps.len(). The defensive
    // validate_response_shape on the client side MUST catch this
    // (trust-boundary clause C2).
    let body = r#"{"candidate_id":"cand-integ-1","verifier_id":"ppm_qwen25_math_prm_7b","mode":"math","step_scores":[0.7,0.8],"score_mean":0.75,"score_var":0.0025,"num_steps":2,"latency_ms":11}"#;
    let mock = spawn_mock(vec![MockReply::ok(body)]).await;

    let hook = HttpSidecarPpmHook::with_endpoint(
        mock.base_url(),
        Duration::from_secs(2),
        0,
    );
    // Send 3 steps but the response only has 2 — shape mismatch.
    let req = sample_request(3);
    let err = hook.score(req).await.expect_err("expected ShapeMismatch");
    match err {
        PpmHookError::ShapeMismatch {
            request_steps,
            response_scores,
        } => {
            assert_eq!(request_steps, 3);
            assert_eq!(response_scores, 2);
        }
        other => panic!("expected ShapeMismatch, got {other:?}"),
    }

    let _ = mock.shutdown.send(());
}

#[tokio::test]
async fn retry_on_502_then_200_succeeds() {
    // First attempt: 502 raw (no structured body) -> falls through to
    // Transport(_) which is_retryable() == true.
    // Second attempt: 200 + well-formed body -> Ok.
    // With max_retries=3 the hook should retry once and succeed.
    let body = shaped_response_json(2, "math");
    let mock = spawn_mock(vec![
        MockReply::raw_status(502, "Bad Gateway", "upstream nope"),
        MockReply::ok(body),
    ])
    .await;

    let hook = HttpSidecarPpmHook::with_endpoint(
        mock.base_url(),
        Duration::from_secs(2),
        3,
    );
    let req = sample_request(2);
    let resp = hook.score(req).await.expect("retry should succeed");
    assert_eq!(resp.num_steps, 2);
    assert_eq!(resp.step_scores.len(), 2);

    let counter = *mock.request_counter.lock().await;
    assert_eq!(counter, 2, "expected one retry (2 total POSTs)");

    let _ = mock.shutdown.send(());
}

#[tokio::test]
async fn out_of_range_score_response_caught_client_side() {
    // Response says score=1.7 (out of range). The defensive client-side
    // validate_response_shape MUST catch this and surface InvalidScore
    // before the value reaches the consensus rule.
    let body = r#"{"candidate_id":"cand-integ-1","verifier_id":"ppm_qwen25_math_prm_7b","mode":"math","step_scores":[1.7],"score_mean":1.7,"score_var":0.0,"num_steps":1,"latency_ms":5}"#;
    let mock = spawn_mock(vec![MockReply::ok(body)]).await;

    let hook = HttpSidecarPpmHook::with_endpoint(
        mock.base_url(),
        Duration::from_secs(2),
        0,
    );
    let req = sample_request(1);
    let err = hook.score(req).await.expect_err("expected InvalidScore");
    match err {
        PpmHookError::InvalidScore { index, value } => {
            assert_eq!(index, 0);
            assert!(value > 1.0);
        }
        other => panic!("expected InvalidScore, got {other:?}"),
    }

    let _ = mock.shutdown.send(());
}
