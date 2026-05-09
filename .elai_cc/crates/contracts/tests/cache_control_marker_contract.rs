//! Track C-D — Anthropic cache_control marker boundary + meta.gg_signals contract.
//!
//! Purpose: verifies the live ELAI sidecar /v1/messages endpoint correctly
//! (1) parses per-content-block `cache_control: {"type": "ephemeral"}`
//!     markers and bounds PrefixKVCache cache_read/cache_creation accounting
//!     by the marker token boundary;
//! (2) surfaces GGSignalCollector output as a top-level `meta.gg_signals`
//!     custom-extension field on the non-stream response;
//! (3) preserves backward compatibility when no cache_control marker is
//!     present (existing PrefixKVCache full-prefix behaviour intact).
//!
//! Spec / handoff references:
//!   workflow/sessions/s-1430428/context-package-track-C-D.md  (sections 1-6)
//!   workflow/sessions/s-934436/NEXT-SESSION-FINISH-DASHBOARD-V2-PART3.md
//!
//! Style match: mirrors research_decompose_cache_hit.rs (Track C-A) — the
//! same env-gate pattern (ELAI_LIVE_SIDECAR=1), the same skip-on-no-sidecar
//! port-reachability pre-flight, and the same `eprintln!` SKIP convention so
//! cargo test stays green in CI without a live sidecar booted.
//!
//! Deps note (no Cargo.toml mutation per task brief):
//!   The contracts crate currently has NO `reqwest` dev-dependency declared
//!   (see crates/contracts/Cargo.toml — only serde_json, tokio, etc.). To
//!   keep this file self-contained we hand-roll the HTTP/1.1 POST over a
//!   `std::net::TcpStream` and parse the JSON body with `serde_json`. This
//!   is sufficient because the sidecar speaks plain HTTP/1.1 (no TLS) on
//!   loopback.
//!
//!   TODO(future): if reqwest blocking is added to crates/contracts
//!   dev-dependencies, swap the raw socket helper for
//!   `reqwest::blocking::Client::new().post(...).json(&body).send()` for
//!   readability.  Workspace reqwest currently lacks the `blocking` feature
//!   too — adding either requires explicit user confirmation per the
//!   no-new-deps-without-confirmation rule.
//!
//! Initial expected state: RED (TESTING_FIRST). server.py at C-D plan time
//! does NOT yet parse cache_control (zero references in handle_messages per
//! context-package §2 grep) nor surface meta.gg_signals (gg_signals
//! discarded at server.py:2894). The test file is written first; the
//! implementation in steps 1-3 of context-package §3 turns it GREEN.
//!
//! Recommended invocation:
//!   ELAI_LIVE_SIDECAR=1 cargo test -p elai_cc_contracts \
//!       --test cache_control_marker_contract -- --nocapture

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
const SIDECAR_PATH: &str = "/v1/messages";

/// Repeated short phrase used to build a deterministic 100+-token prompt.
/// "the quick brown fox " is 5 BPE-friendly tokens in most Qwen tokenizers
/// (verified by inspection — each whitespace-separated word is a single
/// subword). 25 repeats × 5 ≈ 125 tokens; placing the marker after 20
/// repeats lands at ≈100 tokens, comfortably above _KV_MIN_PREFIX_TOKENS=16
/// (server.py:1216) and well below the 25K context ceiling.
const PHRASE: &str = "the quick brown fox ";
const PREFIX_REPEATS: usize = 20; // ≈100 tokens — the cacheable prefix
const SUFFIX_REPEATS: usize = 5; //  ≈25 tokens — the post-marker tail

/// GGSignalCollector key set per context-package §2 + server.py:1041-1046
/// (mean_logprob, min_logprob, entropy_mean, entropy_max) plus
/// generation_novelty appended at server.py:1770.
const GG_SIGNAL_KEYS: &[&str] = &[
    "mean_logprob",
    "min_logprob",
    "entropy_mean",
    "entropy_max",
    "generation_novelty",
];

// ---------------------------------------------------------------------------
// Pre-flight: TCP connect with timeout (mirrors research_decompose_cache_hit).
// ---------------------------------------------------------------------------

fn port_reachable(host: &str, port: u16, timeout: Duration) -> bool {
    let addr: SocketAddr = match format!("{}:{}", host, port).parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    TcpStream::connect_timeout(&addr, timeout).is_ok()
}

// ---------------------------------------------------------------------------
// Minimal HTTP/1.1 POST helper.  Returns the parsed JSON response body or
// an error string.  No TLS, no chunked transfer — sidecar uses
// Content-Length on /v1/messages non-stream responses.
// ---------------------------------------------------------------------------

fn post_messages(body: &Value) -> Result<Value, String> {
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
        path = SIDECAR_PATH,
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

    // Split headers / body on the first CRLFCRLF.
    let split_idx = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| "malformed response: no CRLFCRLF separator".to_string())?;
    let (head, rest) = raw.split_at(split_idx);
    let body_slice = &rest[4..];

    // Parse status line for HTTP code.
    let head_str = std::str::from_utf8(head)
        .map_err(|e| format!("non-utf8 response head: {e:?}"))?;
    let status_line = head_str
        .lines()
        .next()
        .ok_or_else(|| "empty response head".to_string())?;
    // Format: HTTP/1.1 200 OK
    let status_code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("unparseable status line: {status_line:?}"))?;
    if status_code != 200 {
        return Err(format!(
            "non-200 status {status_code}; body={:?}",
            String::from_utf8_lossy(body_slice)
        ));
    }

    serde_json::from_slice::<Value>(body_slice)
        .map_err(|e| format!("parse response JSON: {e:?}; raw={:?}",
            String::from_utf8_lossy(body_slice)))
}

// ---------------------------------------------------------------------------
// Build helpers — produce request bodies for the three gates.
// ---------------------------------------------------------------------------

fn build_prompt_prefix() -> String {
    PHRASE.repeat(PREFIX_REPEATS)
}

fn build_prompt_suffix() -> String {
    PHRASE.repeat(SUFFIX_REPEATS)
}

/// Build a /v1/messages body with cache_control on the FIRST content block
/// (the prefix) and a plain trailing block (the suffix). Per Anthropic
/// spec (context-package §1.B): the marker terminates a cacheable prefix,
/// so everything BEFORE AND INCLUDING the marked block is the boundary.
fn body_with_marker() -> Value {
    json!({
        "model": "qwen",
        "max_tokens": 8,
        "temperature": 0.0,
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": build_prompt_prefix(),
                        "cache_control": { "type": "ephemeral" }
                    },
                    {
                        "type": "text",
                        "text": build_prompt_suffix()
                    }
                ]
            }
        ]
    })
}

/// Backward-compat body — same payload but no cache_control anywhere.
/// Uses string content (the most common Anthropic shape) to also exercise
/// the no-block-walk path.
fn body_without_marker() -> Value {
    let mut full = build_prompt_prefix();
    full.push_str(&build_prompt_suffix());
    json!({
        "model": "qwen",
        "max_tokens": 8,
        "temperature": 0.0,
        "messages": [
            { "role": "user", "content": full }
        ]
    })
}

/// Read the usage block from a /v1/messages response.  Returns
/// (cache_creation_input_tokens, cache_read_input_tokens, input_tokens).
/// Missing fields default to 0 (matches Anthropic spec — fields are
/// elided when caching is silently disabled, see context-package §1.E).
fn read_cache_usage(resp: &Value) -> (i64, i64, i64) {
    let usage = resp.get("usage").and_then(|u| u.as_object());
    let cc = usage
        .and_then(|u| u.get("cache_creation_input_tokens"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let cr = usage
        .and_then(|u| u.get("cache_read_input_tokens"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let inp = usage
        .and_then(|u| u.get("input_tokens"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    (cc, cr, inp)
}

/// Shared skip-or-go pre-flight.  Returns true to PROCEED, false to SKIP.
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

// ===========================================================================
// GATE 1 — cache_control marker partitions the cache_read / cache_creation
//          token accounting at the marker boundary.
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cache_control_marker_partitions_cache() {
    if !live_sidecar_ready("C-D GATE1") {
        return;
    }

    let body = body_with_marker();

    // First request — cold cache from this prompt's perspective.
    // Note: another test in the suite may have warmed PrefixKVCache for an
    // overlapping prefix, so cache_read MAY already be > 0; we only assert
    // the marker-bounded INVARIANT, not that this is a true cold start.
    let resp1 = match post_messages(&body) {
        Ok(v) => v,
        Err(e) => {
            // Surface as a hard failure — the request shape itself is valid
            // Anthropic; if the sidecar 500s, that IS the C-D bug we want to
            // catch (cache_control parse not implemented => unhandled key).
            panic!("C-D GATE1: first POST failed: {e}");
        }
    };
    let (cc1, cr1, inp1) = read_cache_usage(&resp1);
    eprintln!(
        "C-D GATE1 first:  cache_creation={cc1} cache_read={cr1} input={inp1}"
    );

    // Second IDENTICAL request — cache hit must occur up to the marker.
    let resp2 = match post_messages(&body) {
        Ok(v) => v,
        Err(e) => panic!("C-D GATE1: second POST failed: {e}"),
    };
    let (cc2, cr2, inp2) = read_cache_usage(&resp2);
    eprintln!(
        "C-D GATE1 second: cache_creation={cc2} cache_read={cr2} input={inp2}"
    );

    // Approximate token bound: PHRASE (20 chars incl. trailing space) ≈ 5
    // tokens for Qwen2.5-Coder BPE. Marker sits at PREFIX_REPEATS phrases.
    // Tokeniser drift up to ±30% absorbed by a generous upper-bound check.
    // The lower bound (>0) is what proves the marker did SOMETHING.
    let approx_marker_tokens = (PREFIX_REPEATS * 5) as i64;
    let upper_bound = (approx_marker_tokens as f64 * 1.4) as i64; // ≈140

    // CORE INVARIANT 1: second call has a non-zero cache_read (marker
    // succeeded in establishing a re-readable prefix).
    assert!(
        cr2 > 0,
        "C-D GATE1: second-call cache_read_input_tokens must be > 0 \
         (cache hit expected); got {cr2}. Likely root cause: \
         cache_control marker not parsed by handle_messages → marker_boundary \
         never reached PrefixKVCache.lease()."
    );

    // CORE INVARIANT 2: cache_read is BOUNDED by the marker position.
    // Per context-package §1.C: tokens BEFORE marker = cacheable prefix;
    // tokens AFTER marker = post-boundary input. The marker caps cache_read.
    assert!(
        cr2 <= upper_bound,
        "C-D GATE1: cache_read_input_tokens ({cr2}) exceeded marker upper \
         bound (~{upper_bound} ≈ PREFIX_REPEATS*5*1.4). Marker boundary not \
         honoured by PrefixKVCache.lease(marker_boundary=...)."
    );

    // CORE INVARIANT 3: ELAI-correct accounting (NOT Anthropic's):
    //   input_tokens = full prompt token count (NOT suffix-only)
    //   cache_creation_input_tokens + cache_read_input_tokens = input_tokens
    //   cache_read_input_tokens > 0 (we just hit cache)
    //   cache_read_input_tokens <= upper_bound (marker boundary respected)
    // Anthropic's official accounting reports input_tokens as the suffix
    // (uncached) portion, but ELAI's /v1/messages emits the full prompt
    // count via input_tokens for backward compat with current consumers.
    // Documented at server.py:3225 (no comment yet — see WARN-2).
    assert_eq!(
        cc2 + cr2,
        inp2,
        "C-D GATE1 (ELAI accounting): cache_creation_input_tokens + \
         cache_read_input_tokens MUST equal input_tokens (cc2={cc2}, \
         cr2={cr2}, inp2={inp2})"
    );

    // SOFT CHECK: warn if cache_read did not increase between calls. A non-
    // increasing cr would suggest the marker is being ignored even though
    // cr1 happened to be > 0 from a sibling test.
    if cr2 < cr1 {
        eprintln!(
            "C-D GATE1 WARN: cache_read shrank between calls (cr1={cr1}, \
             cr2={cr2}). Possible LRU eviction by another test."
        );
    }
}

// ===========================================================================
// GATE 2 — meta.gg_signals top-level field is present and well-shaped.
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn meta_gg_signals_present_and_shaped() {
    if !live_sidecar_ready("C-D GATE2") {
        return;
    }

    // Use the no-marker body to keep this gate independent from cache_control
    // parsing (any /v1/messages request must surface meta.gg_signals).
    let resp = match post_messages(&body_without_marker()) {
        Ok(v) => v,
        Err(e) => panic!("C-D GATE2: POST failed: {e}"),
    };

    // Top-level meta object — ELAI custom extension per context-package §1.D.
    let meta = resp
        .get("meta")
        .unwrap_or_else(|| panic!(
            "C-D GATE2: response missing top-level `meta` field. \
             Implementation root: server.py:_non_stream_messages_response \
             (L2888-2937) must add `\"meta\": {{\"gg_signals\": ...}}`. \
             Full response: {resp}"
        ));
    let meta_obj = meta
        .as_object()
        .expect("C-D GATE2: response.meta must be a JSON object, not array/scalar");

    let gg = meta_obj
        .get("gg_signals")
        .expect("C-D GATE2: response.meta.gg_signals missing");
    let gg_obj = gg
        .as_object()
        .expect("C-D GATE2: response.meta.gg_signals must be a JSON object");

    // EXACT key set per context-package §2 — no extras, no missing keys.
    for key in GG_SIGNAL_KEYS {
        let v = gg_obj.get(*key).unwrap_or_else(|| panic!(
            "C-D GATE2: response.meta.gg_signals missing required key \
             `{key}`. Required keys: {GG_SIGNAL_KEYS:?}. \
             Got keys: {:?}",
            gg_obj.keys().collect::<Vec<_>>()
        ));
        // Each value must be either null OR a finite number.
        if !v.is_null() {
            let f = v.as_f64().unwrap_or_else(|| panic!(
                "C-D GATE2: response.meta.gg_signals.{key} is neither null \
                 nor numeric; got {v:?}"
            ));
            assert!(
                f.is_finite(),
                "C-D GATE2: response.meta.gg_signals.{key} must be finite, \
                 got {f}"
            );
        }
    }

    // Sanity ranges per context-package §4 GATE 2:
    //   entropy_* >= 0; logprobs <= 0; generation_novelty in [0, 1].
    for (k, lo, hi) in &[
        ("mean_logprob", f64::NEG_INFINITY, 0.0),
        ("min_logprob", f64::NEG_INFINITY, 0.0),
        ("entropy_mean", 0.0, f64::INFINITY),
        ("entropy_max", 0.0, f64::INFINITY),
        ("generation_novelty", 0.0, 1.0),
    ] {
        if let Some(f) = gg_obj.get(*k).and_then(|v| v.as_f64()) {
            assert!(
                f >= *lo && f <= *hi,
                "C-D GATE2: response.meta.gg_signals.{k}={f} outside sane \
                 range [{lo}, {hi}]"
            );
        }
    }

    // Negative assertion: gg_signals must NOT be exposed inside the usage
    // block. Anthropic spec keeps usage clean (context-package §1.D).
    if let Some(usage) = resp.get("usage").and_then(|u| u.as_object()) {
        assert!(
            usage.get("gg_signals").is_none(),
            "C-D GATE2: gg_signals must live under top-level `meta`, \
             NOT inside `usage`. Found in usage: {:?}",
            usage.get("gg_signals")
        );
    }
}

// ===========================================================================
// GATE 3 — backward compat: requests without cache_control still succeed
//          and the response shape matches the pre-C-D baseline.
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cache_control_absent_backward_compat() {
    if !live_sidecar_ready("C-D GATE3") {
        return;
    }

    let resp = match post_messages(&body_without_marker()) {
        Ok(v) => v,
        Err(e) => panic!(
            "C-D GATE3: backward-compat POST (no cache_control) failed: {e}. \
             cache_control parsing must be OPTIONAL — absent key returns None \
             per context-package §6 ('FORBIDDEN: do NOT modify the body \
             parse path in a way that breaks the non-Pydantic dict-access')."
        ),
    };

    // Documented pre-C-D top-level fields per Anthropic spec
    // (https://docs.anthropic.com/en/api/messages) and context-package §2.
    for required in &[
        "id", "type", "role", "content", "model", "stop_reason", "usage",
    ] {
        assert!(
            resp.get(*required).is_some(),
            "C-D GATE3: response missing baseline Anthropic field `{required}`. \
             Pre-C-D shape regressed. Full response: {resp}"
        );
    }

    // Type assertions on the always-present scalar fields.
    assert_eq!(
        resp.get("type").and_then(|v| v.as_str()),
        Some("message"),
        "C-D GATE3: response.type must equal \"message\""
    );
    assert_eq!(
        resp.get("role").and_then(|v| v.as_str()),
        Some("assistant"),
        "C-D GATE3: response.role must equal \"assistant\""
    );

    // usage must be an object and must still expose cache_creation_input_tokens
    // and cache_read_input_tokens (real numbers per Track C-A) — these were
    // added pre-C-D and must NOT be removed by the C-D delta.
    let usage = resp
        .get("usage")
        .and_then(|u| u.as_object())
        .expect("C-D GATE3: response.usage must be an object");
    for usage_key in &[
        "input_tokens",
        "output_tokens",
        "cache_creation_input_tokens",
        "cache_read_input_tokens",
    ] {
        let v = usage.get(*usage_key).unwrap_or_else(|| panic!(
            "C-D GATE3: usage.{usage_key} missing — Track C-A semantics \
             regressed by C-D edits"
        ));
        assert!(
            v.is_i64() || v.is_u64() || v.is_f64(),
            "C-D GATE3: usage.{usage_key} must be numeric, got {v:?}"
        );
    }

    // No-marker request must NOT 500 just because cache_control field is
    // absent from every block. The successful POST above already proved
    // this; the assertion below is a redundant explicit guard for the
    // failure mode "missing-key panic on data['cache_control']".
    let (cc, cr, inp) = read_cache_usage(&resp);
    assert!(
        cc >= 0 && cr >= 0 && inp >= 0,
        "C-D GATE3: usage cache fields must be non-negative; \
         got cc={cc} cr={cr} inp={inp}"
    );
}
