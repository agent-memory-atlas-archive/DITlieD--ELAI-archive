//! Track C-B — constrained JSON decoding via lm-format-enforcer contract.
//!
//! Purpose: verifies the live ELAI sidecar /v1/completions endpoint
//! correctly:
//!   (a) honours `response_format = {"type":"json_schema", ...}` and emits
//!       text that parses as JSON and matches the requested schema;
//!   (b) preserves backward compatibility — requests WITHOUT response_format
//!       behave identically to the pre-C-B baseline (HTTP 200, unconstrained
//!       output);
//!   (c) gracefully handles unsupported response_format types
//!       (e.g. `"regex"`) — server warns + falls back to unconstrained
//!       output, returns HTTP 200, MUST NOT 500.
//!
//! Spec / handoff references:
//!   workflow/sessions/s-1430428/context-package-track-C-B.md  (sections 1-7)
//!   workflow/papers/lm-format-enforcer-interface.md           (paper-ingestor)
//!   workflow/sessions/s-934436/NEXT-SESSION-FINISH-DASHBOARD-V2-PART3.md §5
//!
//! Style match: mirrors cache_control_marker_contract.rs (Track C-D) and
//! research_decompose_cache_hit.rs (Track C-A) — same env-gate pattern
//! (ELAI_LIVE_SIDECAR=1), same skip-on-no-sidecar port-reachability
//! pre-flight, same hand-rolled HTTP/1.1 over `std::net::TcpStream` (the
//! contracts crate has no `reqwest::blocking` dev-dependency and adding one
//! requires user confirmation per the no-new-deps-without-confirmation
//! rule).
//!
//! N=3 trial budget for the success-rate gate (vs the spec's N=10) keeps
//! the live test cheap; the orchestrator-side audit pass repeats at
//! N>=10 per context-package §4 GATE 1 if the cheap pass succeeds.
//!
//! Library-presence gating: lm-format-enforcer is a USER-installed optional
//! dependency (USER-MANUAL-CB-DEPS.md). The sidecar returns HTTP 503 with
//! `{"error":"constrained_decoding_unavailable", ...}` when response_format
//! is set but the library is missing. Gates (a) and (c) detect this 503
//! once via the `ELAI_LMFE_INSTALLED=1` env var: when unset, both gates
//! print SKIP and return early. Gate (b) does not depend on the library
//! and runs whenever ELAI_LIVE_SIDECAR=1 is set.
//!
//! Initial expected state: RED — server.py at C-B plan time does not yet
//! parse response_format. The implementation in context-package §3 +
//! engine.generate signature change (PART3 §5 corrected per paper §40-46
//! to use prefix_allowed_tokens_fn= NOT LogitsProcessorList) turns it
//! GREEN.
//!
//! Recommended invocation (after USER-MANUAL-CB-DEPS.md complete):
//!   ELAI_LIVE_SIDECAR=1 ELAI_LMFE_INSTALLED=1 \
//!     cargo test -p elai_cc_contracts \
//!       --test research_decompose_constrained -- --nocapture

use std::env;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// Sidecar host:port (loopback only — see CLAUDE.md "Remote sidecar" block:
/// the SSH tunnel forwards 127.0.0.1:8100 to the remote GPU box).
const SIDECAR_HOST: &str = "127.0.0.1";
const SIDECAR_PORT: u16 = 8100;
const SIDECAR_COMPLETIONS_PATH: &str = "/v1/completions";
const SIDECAR_MESSAGES_PATH: &str = "/v1/messages";

/// N=3 trials for gate (a). The cheap pass requires >=2/3 successes; the
/// orchestrator audit at N=10 repeats if the cheap pass succeeds. Per
/// context-package §4 GATE 1 the production threshold is 9/10 (>=90%);
/// the cheap-pass `>=2/3` ratio (~66%) is intentionally laxer to absorb
/// single-trial flake without burning sidecar VRAM-time.
const N_TRIALS: usize = 3;
const N_REQUIRED_PASS: usize = 2;

/// Per-call sidecar generation budget. The schema we constrain to is small
/// (two short keys) so a 64-token budget is comfortably above the worst-case
/// Qwen2.5-Coder JSON shape (a few extra whitespace tokens on a slow draw).
const MAX_TOKENS: u32 = 64;

// ---------------------------------------------------------------------------
// Pre-flight: TCP connect with timeout (mirrors cache_control_marker_contract).
// ---------------------------------------------------------------------------

fn port_reachable(host: &str, port: u16, timeout: Duration) -> bool {
    let addr: SocketAddr = match format!("{}:{}", host, port).parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    TcpStream::connect_timeout(&addr, timeout).is_ok()
}

// ---------------------------------------------------------------------------
// Minimal HTTP/1.1 POST helper. Same hand-rolled pattern as
// cache_control_marker_contract.rs::post_messages, generalised over the URL
// path so we can hit /v1/completions here without tripping the no-new-deps
// rule.
//
// Returns Ok((status_code, parsed_body)) on a successful read.  status_code
// surfaces the HTTP code so callers can distinguish 200 / 503 / 500 paths
// without re-parsing the headers.
// ---------------------------------------------------------------------------

fn post_json(path: &str, body: &Value) -> Result<(u16, Value), String> {
    let body_bytes = serde_json::to_vec(body)
        .map_err(|e| format!("serialise request body: {e:?}"))?;
    let addr: SocketAddr = format!("{SIDECAR_HOST}:{SIDECAR_PORT}")
        .parse()
        .map_err(|e| format!("parse sidecar addr: {e:?}"))?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5))
        .map_err(|e| format!("connect sidecar: {e:?}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(180)))
        .map_err(|e| format!("set read timeout: {e:?}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(30)))
        .map_err(|e| format!("set write timeout: {e:?}"))?;

    let request = format!(
        "POST {path} HTTP/1.1\r\n\
         Host: {host}:{port}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {len}\r\n\
         Connection: close\r\n\
         \r\n",
        path = path,
        host = SIDECAR_HOST,
        port = SIDECAR_PORT,
        len = body_bytes.len(),
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("write headers: {e:?}"))?;
    stream
        .write_all(&body_bytes)
        .map_err(|e| format!("write body: {e:?}"))?;
    stream
        .flush()
        .map_err(|e| format!("flush stream: {e:?}"))?;

    let mut raw = Vec::with_capacity(8 * 1024);
    stream
        .read_to_end(&mut raw)
        .map_err(|e| format!("read response: {e:?}"))?;

    let split_idx = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| "malformed response: no CRLFCRLF separator".to_string())?;
    let (head, rest) = raw.split_at(split_idx);
    let body_slice = &rest[4..];

    let head_str = std::str::from_utf8(head)
        .map_err(|e| format!("non-utf8 response head: {e:?}"))?;
    let status_line = head_str
        .lines()
        .next()
        .ok_or_else(|| "empty response head".to_string())?;
    let status_code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("unparseable status line: {status_line:?}"))?;

    // Body MAY be empty on some non-2xx paths — surface the raw body string
    // for callers that need it for assertion messages.
    if body_slice.is_empty() {
        return Ok((status_code, Value::Null));
    }
    let parsed = serde_json::from_slice::<Value>(body_slice).map_err(|e| {
        format!(
            "parse response JSON (status={status_code}): {e:?}; raw={:?}",
            String::from_utf8_lossy(body_slice)
        )
    })?;
    Ok((status_code, parsed))
}

// ---------------------------------------------------------------------------
// Schema fixture — small two-key object the model must satisfy.
// ---------------------------------------------------------------------------

/// JSON schema for the constrained-decoding gate. Two keys, both required,
/// `status` is enum-restricted to keep the constraint tight. The schema is
/// hand-derived (no `schemars` dep — no Rust crate adds per
/// context-package §6 "no new deps without confirmation").
fn step_status_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "step": { "type": "string" },
            "status": { "type": "string", "enum": ["ok", "fail"] }
        },
        "required": ["step", "status"]
    })
}

/// Build the response_format payload per Anthropic / OpenAI-compatible
/// `{"type":"json_schema", "json_schema":{...}}` wrapper.
fn response_format_step_status() -> Value {
    json!({
        "type": "json_schema",
        "json_schema": {
            "name": "step_status",
            "schema": step_status_schema()
        }
    })
}

/// Validate a parsed JSON value against the step_status schema. Hand-rolled
/// (no `jsonschema` dep): asserts top-level is an object with exactly the
/// two required keys present, `step` is a string, and `status` is one of
/// the enum members.
fn matches_step_status_schema(v: &Value) -> bool {
    let obj = match v.as_object() {
        Some(o) => o,
        None => return false,
    };
    let step_ok = obj
        .get("step")
        .and_then(|s| s.as_str())
        .is_some();
    let status_ok = obj
        .get("status")
        .and_then(|s| s.as_str())
        .map(|s| s == "ok" || s == "fail")
        .unwrap_or(false);
    step_ok && status_ok
}

/// Pull the first choice's text out of a /v1/completions response envelope.
/// Returns None if the response shape is unexpected (the per-trial caller
/// counts that as a failure, not a panic).
fn first_choice_text(resp: &Value) -> Option<String> {
    resp.get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("text"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
}

// ---------------------------------------------------------------------------
// Skip-or-go pre-flight. ELAI_LIVE_SIDECAR gates EVERY test (no live sidecar
// in CI). ELAI_LMFE_INSTALLED additionally gates the gates that DEPEND on
// lm-format-enforcer being installed on the remote sidecar.
// ---------------------------------------------------------------------------

fn live_sidecar_ready(gate: &str) -> bool {
    if env::var("ELAI_LIVE_SIDECAR").is_err() {
        eprintln!("{gate} SKIP: ELAI_LIVE_SIDECAR not set");
        return false;
    }
    if !port_reachable(SIDECAR_HOST, SIDECAR_PORT, Duration::from_secs(5)) {
        eprintln!(
            "{gate} SKIP: sidecar not reachable at {SIDECAR_HOST}:{SIDECAR_PORT}"
        );
        return false;
    }
    true
}

fn lmfe_required_ready(gate: &str) -> bool {
    if !live_sidecar_ready(gate) {
        return false;
    }
    if env::var("ELAI_LMFE_INSTALLED").is_err() {
        eprintln!(
            "{gate} SKIP: ELAI_LMFE_INSTALLED not set — \
             lm-format-enforcer must be installed on the sidecar per \
             USER-MANUAL-CB-DEPS.md before running this gate"
        );
        return false;
    }
    true
}

// ===========================================================================
// GATE A — constrained JSON returns valid schema match.
//
// Procedure: N=3 trials with varied prompts, each requesting the
// step_status schema. PASS requires >=2/3 trials where the returned text
// parses as JSON AND matches the schema (top-level object, both keys
// present, status in {"ok","fail"}).
//
// Failure-mode triage:
//   * 503 returned → lm-format-enforcer missing on sidecar, skip via
//     ELAI_LMFE_INSTALLED gate above.
//   * HTTP 200 but parse failure → schema constraint not wired into
//     model.generate() (the prefix_allowed_tokens_fn= kwarg path).
//   * HTTP 200 + JSON parses but schema mismatch → JsonSchemaParser was
//     constructed with None instead of the supplied dict (server.py
//     _build_constrained_prefix_fn extraction bug).
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn constrained_json_returns_valid_schema_match() {
    let gate = "C-B GATE A";
    if !lmfe_required_ready(gate) {
        return;
    }

    // Three varied prompts; the schema is identical across trials.  All
    // three should drive the model toward emitting `{"step":"...", "status":"ok|fail"}`.
    let prompts = [
        "Emit a JSON object describing build step 1 of a Rust project.",
        "Return JSON with the install status of a Python dependency.",
        "Output JSON for a passing unit test summary.",
    ];

    let mut passes = 0usize;
    let mut failures: Vec<String> = Vec::new();

    for (i, prompt) in prompts.iter().enumerate() {
        let body = json!({
            "prompt": prompt,
            "max_tokens": MAX_TOKENS,
            "temperature": 0.0,
            "response_format": response_format_step_status(),
        });
        let (status, resp) = match post_json(SIDECAR_COMPLETIONS_PATH, &body) {
            Ok(t) => t,
            Err(e) => {
                failures.push(format!("trial {i}: transport failure: {e}"));
                continue;
            }
        };
        if status != 200 {
            failures.push(format!(
                "trial {i}: non-200 status {status}; body={resp}"
            ));
            continue;
        }
        let text = match first_choice_text(&resp) {
            Some(t) => t,
            None => {
                failures.push(format!(
                    "trial {i}: response missing choices[0].text; resp={resp}"
                ));
                continue;
            }
        };
        // Trim — the model may emit leading/trailing whitespace tokens
        // that lm-format-enforcer permits per default
        // CharacterLevelParserConfig (paper §78).
        let trimmed = text.trim();
        let parsed = match serde_json::from_str::<Value>(trimmed) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!(
                    "trial {i}: JSON parse failure: {e}; text={trimmed:?}"
                ));
                continue;
            }
        };
        if !matches_step_status_schema(&parsed) {
            failures.push(format!(
                "trial {i}: parsed JSON does not match schema; parsed={parsed}"
            ));
            continue;
        }
        passes += 1;
        eprintln!("{gate} trial {i} PASS: {trimmed}");
    }

    eprintln!(
        "{gate} summary: {passes}/{N_TRIALS} pass (need >={N_REQUIRED_PASS})"
    );
    for f in &failures {
        eprintln!("{gate} failure: {f}");
    }
    assert!(
        passes >= N_REQUIRED_PASS,
        "{gate}: only {passes}/{N_TRIALS} trials produced schema-valid JSON \
         (required >= {N_REQUIRED_PASS}). Failures: {failures:?}. \
         Likely root cause: prefix_allowed_tokens_fn= kwarg not threaded \
         through model.generate(), OR JsonSchemaParser invoked with None \
         instead of the supplied dict (see context-package §3)."
    );
}

// ===========================================================================
// GATE B — backward compat: requests WITHOUT response_format work as before.
//
// This gate does NOT depend on lm-format-enforcer being installed (the
// pre-C-B code path is unchanged when response_format is absent). It is
// gated only by ELAI_LIVE_SIDECAR.
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn response_format_absent_backward_compat() {
    let gate = "C-B GATE B";
    if !live_sidecar_ready(gate) {
        return;
    }

    // Identical to a pre-C-B /v1/completions request — no response_format
    // key. Server must accept and return HTTP 200 with the OpenAI-compatible
    // envelope.
    let body = json!({
        "prompt": "Say hello in one short sentence.",
        "max_tokens": MAX_TOKENS,
        "temperature": 0.0,
    });

    let (status, resp) = match post_json(SIDECAR_COMPLETIONS_PATH, &body) {
        Ok(t) => t,
        Err(e) => panic!("{gate}: POST failed: {e}"),
    };
    assert_eq!(
        status, 200,
        "{gate}: pre-C-B request without response_format must return 200; \
         got {status}; body={resp}"
    );

    let text = first_choice_text(&resp).unwrap_or_else(|| {
        panic!(
            "{gate}: response missing choices[0].text — backward-compat \
             envelope shape regressed; resp={resp}"
        )
    });
    assert!(
        !text.is_empty(),
        "{gate}: response choices[0].text must be non-empty for an \
         unconstrained completion; resp={resp}"
    );

    // Negative assertion: text MUST NOT be required to parse as JSON in
    // the unconstrained path. Asserting on a substring would couple the
    // test to model output; we settle for non-emptiness which is stable.
}

// ===========================================================================
// GATE C — invalid schema type warns, does NOT 500.
//
// Sets `response_format.type = "regex"` (an unsupported type per
// context-package §3b — only "json_schema" is honoured). Server must
// warn-and-ignore: HTTP 200 with unconstrained output. A 500 here means
// the server raised on the unsupported type instead of falling back.
//
// Depends on lm-format-enforcer being installed because the warn-and-ignore
// path lives inside _build_constrained_prefix_fn — when LMFE is missing,
// the upstream 503 short-circuit fires before that helper runs. Gated via
// ELAI_LMFE_INSTALLED.
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn invalid_schema_type_warns_not_500() {
    let gate = "C-B GATE C";
    if !lmfe_required_ready(gate) {
        return;
    }

    let body = json!({
        "prompt": "Anything.",
        "max_tokens": MAX_TOKENS,
        "temperature": 0.0,
        "response_format": {
            "type": "regex",
            "regex": "[A-Z]+"
        }
    });

    let (status, resp) = match post_json(SIDECAR_COMPLETIONS_PATH, &body) {
        Ok(t) => t,
        Err(e) => panic!("{gate}: POST failed: {e}"),
    };

    // Critical invariant: server MUST NOT 500. 200 (warn-and-ignore) is
    // the documented behaviour; 400 would also be acceptable per
    // context-package §4 GATE 3a if a future track elects to harden
    // validation, but 500 indicates the unsupported type bubbled out as
    // an unhandled exception.
    assert_ne!(
        status, 500,
        "{gate}: unsupported response_format.type='regex' triggered HTTP 500; \
         server must warn-and-ignore instead. resp={resp}"
    );
    // Current implementation: warn + ignore + 200 with unconstrained output.
    assert_eq!(
        status, 200,
        "{gate}: expected 200 (warn-and-ignore path); got {status}; body={resp}. \
         If this changes to 400 in a future track, update this assertion to \
         match the new validation policy in context-package §4 GATE 3a."
    );
    // Sanity: the unconstrained envelope must still surface a choices[].text.
    let text = first_choice_text(&resp).unwrap_or_else(|| {
        panic!(
            "{gate}: warn-and-ignore path must still return a valid \
             completions envelope; resp={resp}"
        )
    });
    assert!(
        !text.is_empty(),
        "{gate}: warn-and-ignore path returned empty text; resp={resp}"
    );
}

// ===========================================================================
// GATE D — streaming + response_format returns constrained SSE
// (Phase B s-1976592 — replaces the prior unconditional-503 gate).
//
// Procedure: POST /v1/messages with stream=true AND response_format set.
// Expected (LMFE installed): HTTP 200, Content-Type `text/event-stream`,
// at least one non-empty SSE `data:` event with text content. The
// prefix_allowed_tokens_fn is now wired into generate_streaming so output
// honours the JSON schema as it streams.
//
// LMFE-absent case is exercised by the non-stream Gate B/C path: when LMFE
// is missing, /v1/messages returns 503 _ERR_CONSTRAINED_DECODING_UNAVAILABLE
// before even reaching the stream branch (handler gate at server.py L2809).
// No second live-sidecar fn is needed for that branch — pythondev keeps
// LMFE permanently installed.
//
// Gated by lmfe_required_ready (ELAI_LIVE_SIDECAR + ELAI_LMFE_INSTALLED).
// Reads raw bytes (post_sse helper) to inspect Content-Type header and SSE
// payload — post_json's serde parsing would reject the SSE body.
// ===========================================================================

fn post_sse(path: &str, body: &Value) -> Result<(u16, String, Vec<u8>), String> {
    let body_bytes = serde_json::to_vec(body)
        .map_err(|e| format!("serialise request body: {e:?}"))?;
    let addr: SocketAddr = format!("{SIDECAR_HOST}:{SIDECAR_PORT}")
        .parse()
        .map_err(|e| format!("parse sidecar addr: {e:?}"))?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5))
        .map_err(|e| format!("connect sidecar: {e:?}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(180)))
        .map_err(|e| format!("set read timeout: {e:?}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(30)))
        .map_err(|e| format!("set write timeout: {e:?}"))?;

    let request = format!(
        "POST {path} HTTP/1.1\r\n\
         Host: {host}:{port}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {len}\r\n\
         Connection: close\r\n\
         \r\n",
        path = path,
        host = SIDECAR_HOST,
        port = SIDECAR_PORT,
        len = body_bytes.len(),
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("write headers: {e:?}"))?;
    stream
        .write_all(&body_bytes)
        .map_err(|e| format!("write body: {e:?}"))?;
    stream
        .flush()
        .map_err(|e| format!("flush stream: {e:?}"))?;

    let mut raw = Vec::with_capacity(64 * 1024);
    stream
        .read_to_end(&mut raw)
        .map_err(|e| format!("read response: {e:?}"))?;

    let split_idx = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| "malformed response: no CRLFCRLF separator".to_string())?;
    let (head, rest) = raw.split_at(split_idx);
    let body_slice = rest[4..].to_vec();

    let head_str = std::str::from_utf8(head)
        .map_err(|e| format!("non-utf8 response head: {e:?}"))?;
    let mut lines = head_str.lines();
    let status_line = lines
        .next()
        .ok_or_else(|| "empty response head".to_string())?;
    let status_code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("unparseable status line: {status_line:?}"))?;
    let head_lc = head_str.to_ascii_lowercase();

    Ok((status_code, head_lc, body_slice))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stream_response_format_constrained_sse_when_lmfe_installed() {
    let gate = "C-B GATE D";
    if !lmfe_required_ready(gate) {
        return;
    }

    let body = json!({
        "model": "elai-local",
        "max_tokens": MAX_TOKENS,
        "stream": true,
        "messages": [
            { "role": "user", "content": "Emit JSON describing a build step." }
        ],
        "response_format": response_format_step_status(),
    });

    let (status, head_lc, body_bytes) = match post_sse(SIDECAR_MESSAGES_PATH, &body) {
        Ok(t) => t,
        Err(e) => panic!("{gate}: POST failed: {e}"),
    };

    let body_str = String::from_utf8_lossy(&body_bytes).to_string();
    assert_eq!(
        status, 200,
        "{gate}: stream=true + response_format with LMFE installed must return 200; \
         got {status}; head={head_lc}; body={body_str}. Root cause likely: \
         server.py generate_streaming missing prefix_allowed_tokens_fn wiring \
         OR handle_messages stream branch still issuing legacy 503."
    );
    assert!(
        head_lc.contains("text/event-stream"),
        "{gate}: response Content-Type must be text/event-stream; head={head_lc}"
    );

    let mut data_payloads: Vec<&str> = Vec::new();
    for line in body_str.split('\n') {
        let trimmed = line.trim_end_matches('\r');
        if let Some(rest) = trimmed.strip_prefix("data:") {
            let payload = rest.trim();
            if !payload.is_empty() {
                data_payloads.push(payload);
            }
        }
    }
    assert!(
        !data_payloads.is_empty(),
        "{gate}: SSE stream emitted no non-empty data: events; body={body_str}"
    );

    // Audit-tightened (Phase B WARN-1): require at least one content_block_delta
    // event with non-empty delta.text. message_start / content_block_start alone
    // do NOT prove generation tokens were emitted — they're sent by the SSE
    // generator before the streamer drain loop starts. Requiring delta.text
    // proves model.generate produced at least one token under the prefix_fn
    // constraint (or fallback unconstrained streaming if schema invalid).
    let any_delta_text = data_payloads.iter().any(|p| {
        match serde_json::from_str::<Value>(p) {
            Ok(v) => v
                .get("delta")
                .and_then(|d| d.get("text"))
                .and_then(|t| t.as_str())
                .is_some_and(|t| !t.is_empty()),
            Err(_) => false,
        }
    });
    assert!(
        any_delta_text,
        "{gate}: SSE stream had data: lines but no content_block_delta event \
         carried non-empty delta.text — generation produced zero tokens, the \
         prefix_fn wiring is unverified; payloads={data_payloads:?}"
    );
}

// ---------------------------------------------------------------------------
// PART7 §3.3 / §5.A-3: backward-compat gate.
//
// Phase B (PART6) added a `response_format` kwarg to `generate_streaming` and
// `_stream_messages_response`. This test asserts the no-response_format path
// is unchanged: stream=true with response_format absent must still emit
// normal SSE flow (>=1 content_block_delta with non-empty delta.text). Gate
// is `live_sidecar_ready` — LMFE is irrelevant when response_format is None,
// so this test runs whenever ELAI_LIVE_SIDECAR=1, regardless of
// ELAI_LMFE_INSTALLED.
//
// Catches the regression class where the new prefix_fn wiring accidentally
// short-circuits the no-constraint path or fails to thread None through.
// ---------------------------------------------------------------------------
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stream_with_no_response_format_unchanged_sse() {
    let gate = "PART7 §3.3 GATE";
    if !live_sidecar_ready(gate) {
        return;
    }

    let body = json!({
        "model": "elai-local",
        "max_tokens": MAX_TOKENS,
        "stream": true,
        "messages": [
            { "role": "user", "content": "Reply with exactly: hello world." }
        ],
    });

    let (status, head_lc, body_bytes) = match post_sse(SIDECAR_MESSAGES_PATH, &body) {
        Ok(t) => t,
        Err(e) => panic!("{gate}: POST failed: {e}"),
    };

    let body_str = String::from_utf8_lossy(&body_bytes).to_string();
    assert_eq!(
        status, 200,
        "{gate}: stream=true with NO response_format must return 200 (backward-compat); \
         got {status}; head={head_lc}; body={body_str}. Root cause likely: Phase B \
         response_format=None thread-through is broken in generate_streaming or \
         _stream_messages_response."
    );
    assert!(
        head_lc.contains("text/event-stream"),
        "{gate}: response Content-Type must be text/event-stream; head={head_lc}"
    );

    let mut data_payloads: Vec<&str> = Vec::new();
    for line in body_str.split('\n') {
        let trimmed = line.trim_end_matches('\r');
        if let Some(rest) = trimmed.strip_prefix("data:") {
            let payload = rest.trim();
            if !payload.is_empty() {
                data_payloads.push(payload);
            }
        }
    }
    assert!(
        !data_payloads.is_empty(),
        "{gate}: SSE stream emitted no non-empty data: events; body={body_str}"
    );

    let any_delta_text = data_payloads.iter().any(|p| {
        match serde_json::from_str::<Value>(p) {
            Ok(v) => v
                .get("delta")
                .and_then(|d| d.get("text"))
                .and_then(|t| t.as_str())
                .is_some_and(|t| !t.is_empty()),
            Err(_) => false,
        }
    });
    assert!(
        any_delta_text,
        "{gate}: SSE stream produced no content_block_delta with non-empty delta.text \
         — Phase B no-response_format path emits zero generation tokens; payloads={data_payloads:?}"
    );
}
