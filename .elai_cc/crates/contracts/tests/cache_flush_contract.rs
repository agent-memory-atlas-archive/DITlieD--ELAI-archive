//! PART7 §5.A-5 — POST /v1/cache/flush contract.
//!
//! Purpose: verify the live ELAI sidecar exposes a /v1/cache/flush endpoint
//! that:
//!   (a) returns HTTP 200 with JSON `{"flushed": <bool>, "n_entries_cleared": <int>}`;
//!   (b) honours the operator-driven cold-start contract: after a successful
//!       flush, the next prefix-cache-bench cold/warm timing pair must show
//!       a true cold start (the lease() call returns no match for any
//!       previously admitted prefix).
//!
//! Scope of THIS test (offline-safe by design):
//!   only assertion (a) — the JSON-shape contract. Assertion (b) is a
//!   coupled bench-level invariant that prefix_cache_bench.py exercises;
//!   this test does NOT replicate the bench. The flush endpoint MUST be
//!   safe to call when the cache is empty or disabled (turboquant active /
//!   max_entries=0 / kv_cache_size=0): that path returns
//!   `{"flushed": false, "n_entries_cleared": 0}` with status 200, NOT 503
//!   — the operator should be able to call /v1/cache/flush blindly between
//!   bench runs without conditional handling.
//!
//! Style: matches research_decompose_constrained.rs — same hand-rolled
//! HTTP/1.1 over std::net::TcpStream, no new dev-deps, same SIDECAR_HOST /
//! SIDECAR_PORT loopback constants, same env-gated skip semantics.
//!
//! Gate: `ELAI_LIVE_SIDECAR=1`. LMFE is irrelevant — the flush endpoint has
//! no LMFE coupling — so this test runs whenever the sidecar is reachable.
//!
//! Recommended invocation:
//!   ELAI_LIVE_SIDECAR=1 \
//!     cargo test -p elai_cc_contracts \
//!       --test cache_flush_contract -- --nocapture

use std::env;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// Fixtures (mirror research_decompose_constrained.rs — no shared util crate).
// ---------------------------------------------------------------------------

const SIDECAR_HOST: &str = "127.0.0.1";
const SIDECAR_PORT: u16 = 8100;
const SIDECAR_FLUSH_PATH: &str = "/v1/cache/flush";

// ---------------------------------------------------------------------------
// Skip-or-go pre-flight. Inlined per PART7 context-package: no util crate.
// ---------------------------------------------------------------------------

fn port_reachable(host: &str, port: u16, timeout: Duration) -> bool {
    let addr: SocketAddr = match format!("{}:{}", host, port).parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    TcpStream::connect_timeout(&addr, timeout).is_ok()
}

fn live_sidecar_ready(gate: &str) -> bool {
    if env::var("ELAI_LIVE_SIDECAR").ok().as_deref() != Some("1") {
        eprintln!("{gate} SKIP: ELAI_LIVE_SIDECAR != 1");
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

// ---------------------------------------------------------------------------
// Minimal HTTP/1.1 POST helper. Lifted verbatim (with one-line comment
// adjustments) from research_decompose_constrained.rs::post_json — no
// shared util crate exists.
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
        .set_read_timeout(Some(Duration::from_secs(30)))
        .map_err(|e| format!("set read timeout: {e:?}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
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

    let mut raw = Vec::with_capacity(4 * 1024);
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
// Test: shape contract for /v1/cache/flush.
// ---------------------------------------------------------------------------

#[test]
fn cache_flush_returns_200_with_shape_contract() {
    let gate = "PART7 §5.A-5 GATE";
    if !live_sidecar_ready(gate) {
        return;
    }

    // Empty JSON body — endpoint takes no parameters. Anthropic-style POST
    // semantics still require a Content-Length, so an empty `{}` is the
    // canonical zero-arg payload.
    let body = json!({});

    let (status, parsed) = match post_json(SIDECAR_FLUSH_PATH, &body) {
        Ok(t) => t,
        Err(e) => panic!("{gate}: POST failed: {e}"),
    };

    assert_eq!(
        status, 200,
        "{gate}: /v1/cache/flush must return 200 even when cache is empty/disabled \
         (operators should be able to call this blindly between bench runs); \
         got {status}; body={parsed}"
    );

    let obj = parsed.as_object().unwrap_or_else(|| {
        panic!("{gate}: response body must be a JSON object; got {parsed}")
    });

    let flushed = obj
        .get("flushed")
        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| {
            panic!(
                "{gate}: response.flushed must be a bool; got {:?}",
                obj.get("flushed")
            )
        });
    let _ = flushed; // value is contract-defined; presence + type is the gate

    let n_cleared = obj
        .get("n_entries_cleared")
        .and_then(|v| v.as_i64())
        .unwrap_or_else(|| {
            panic!(
                "{gate}: response.n_entries_cleared must be an integer; got {:?}",
                obj.get("n_entries_cleared")
            )
        });
    assert!(
        n_cleared >= 0,
        "{gate}: response.n_entries_cleared must be non-negative; got {n_cleared}"
    );
}

// ---------------------------------------------------------------------------
// Test: idempotent. Two consecutive flushes both return 200; the second
// must report n_entries_cleared == 0 (cache is empty after the first).
// ---------------------------------------------------------------------------

#[test]
fn cache_flush_is_idempotent() {
    let gate = "PART7 §5.A-5 IDEMPOTENT";
    if !live_sidecar_ready(gate) {
        return;
    }

    let body = json!({});

    let (s1, _p1) = match post_json(SIDECAR_FLUSH_PATH, &body) {
        Ok(t) => t,
        Err(e) => panic!("{gate}: first POST failed: {e}"),
    };
    assert_eq!(s1, 200, "{gate}: first flush must return 200");

    let (s2, p2) = match post_json(SIDECAR_FLUSH_PATH, &body) {
        Ok(t) => t,
        Err(e) => panic!("{gate}: second POST failed: {e}"),
    };
    assert_eq!(s2, 200, "{gate}: second flush must return 200");

    let n_cleared = p2
        .as_object()
        .and_then(|o| o.get("n_entries_cleared"))
        .and_then(|v| v.as_i64())
        .unwrap_or_else(|| panic!("{gate}: second flush body malformed: {p2}"));
    assert_eq!(
        n_cleared, 0,
        "{gate}: second consecutive flush must clear 0 entries (cache was emptied \
         by the first); got n_entries_cleared={n_cleared}; body={p2}"
    );
}
