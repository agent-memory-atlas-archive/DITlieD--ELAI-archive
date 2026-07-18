//! `SidecarRuntime` — adapter that POSTs Anthropic-shaped messages to
//! the local sidecar (`python/elai_inference/server.py`).
//!
//! The sidecar already speaks the Anthropic `/v1/messages` protocol
//! (server.py:3016, B-37 decisions.md item 7). We deliberately bypass
//! the existing `SidecarProvider::complete()` path — that one takes a
//! plain `&str` prompt + `OllamaConfig` and returns `Vec<String>`,
//! which would force a lossy round-trip back from the multi-message
//! envelope. The B-37A wrapper talks HTTP directly.
//!
//! Health check: GET `/health` (server.py:2558).
//!
//! Scope of B-37A:
//! - Non-streaming `complete` builds an `InputMessage[]` request and
//!   reads the `MessageResponse`.
//! - `stream` emits a single error chunk (streaming through the local
//!   sidecar is deferred to B-37C, when we also wire up
//!   `MessageStream` token translation).
//! - `health_check` GETs `/health`; non-2xx -> `Unavailable`.

use std::time::Duration;

use elai_core::clock::{ClockProvider, OsClock};
use futures_util::stream::{self, BoxStream, StreamExt};
use reqwest::Response;
use serde_json::Value;

use crate::types::{InputContentBlock, InputMessage, MessageRequest, MessageResponse, SystemBlock};

use super::types::{
    unavailable, CompletionConfig, CompletionResponse, Message, MessageRole, RuntimeAdapter,
    RuntimeAdapterError, RuntimeBackend, RuntimeCapabilities, RuntimeFactoryConfig, RuntimeFuture,
    RuntimeHealth, RuntimeHealthFuture, StreamChunk,
};

const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 120;
const DEFAULT_HEALTH_TIMEOUT_SECS: u64 = 5;
const DEFAULT_MAX_TOKENS: u32 = 512;
const DEFAULT_MODEL: &str = "sidecar-default";
/// Matches `crate::sidecar_client::DEFAULT_SIDECAR_URL`. Inlined so the
/// runtime module compiles regardless of the `sidecar` cargo feature
/// (`sidecar_client` is feature-gated; the multi-runtime adapter must
/// not be).
const DEFAULT_SIDECAR_URL: &str = "http://localhost:8100";

/// HTTP-backed runtime adapter for the local sidecar.
#[derive(Debug, Clone)]
pub struct SidecarRuntime {
    server_url: String,
    model: String,
    http: reqwest::Client,
    health_http: reqwest::Client,
    request_timeout: Duration,
    max_tokens: u32,
}

impl SidecarRuntime {
    /// Build the runtime from a `RuntimeFactoryConfig`. The
    /// `sidecar`/`base_url`/`model` fields are honoured in that order.
    pub fn from_factory_config(cfg: &RuntimeFactoryConfig) -> Result<Self, RuntimeAdapterError> {
        let sidecar = cfg.sidecar.clone().unwrap_or_default();

        let server_url = sidecar
            .server_url
            .clone()
            .or_else(|| cfg.base_url.clone())
            .unwrap_or_else(|| DEFAULT_SIDECAR_URL.to_string());

        let model = cfg
            .model
            .clone()
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());

        let request_timeout = sidecar
            .request_timeout_secs
            .map(Duration::from_secs)
            .unwrap_or_else(|| Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS));

        let max_tokens = sidecar.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);

        let http = reqwest::Client::builder()
            .timeout(request_timeout)
            .build()
            .map_err(|e| RuntimeAdapterError::Network(e.to_string()))?;
        let health_http = reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_HEALTH_TIMEOUT_SECS))
            .build()
            .map_err(|e| RuntimeAdapterError::Network(e.to_string()))?;

        Ok(Self {
            server_url,
            model,
            http,
            health_http,
            request_timeout,
            max_tokens,
        })
    }

    fn build_request(&self, messages: Vec<Message>, config: &CompletionConfig) -> MessageRequest {
        let model = config
            .model
            .clone()
            .unwrap_or_else(|| self.model.clone());

        let mut system = None;
        if let Some(text) = &config.system {
            system = Some(vec![SystemBlock::text(text.clone())]);
        }

        let mut input_messages = Vec::with_capacity(messages.len());
        for msg in messages {
            // The sidecar's /v1/messages accepts only user/assistant
            // roles in `messages`; system goes through `system`, and
            // tool turns are deferred to B-37B. We coerce other roles
            // into user content so we never silently drop a message.
            let role = match msg.role {
                MessageRole::Assistant => "assistant",
                MessageRole::User | MessageRole::System | MessageRole::Tool => "user",
            };
            input_messages.push(InputMessage {
                role: role.to_string(),
                content: vec![InputContentBlock::Text {
                    text: msg.content,
                    cache_control: None,
                }],
            });
        }

        let max_tokens = if config.max_tokens == 0 {
            self.max_tokens
        } else {
            config.max_tokens
        };

        MessageRequest {
            model,
            max_tokens,
            messages: input_messages,
            system,
            tools: None,
            tool_choice: None,
            stream: false,
            seed: config.seed,
            // I-26: effort is a cloud-only field; sidecar never uses it.
            effort: None,
            chat_template_kwargs: config.chat_template_kwargs.clone(),
        }
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}{}", self.server_url.trim_end_matches('/'), path)
    }
}

impl RuntimeAdapter for SidecarRuntime {
    fn backend(&self) -> RuntimeBackend {
        RuntimeBackend::Sidecar
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        // Conservative — the sidecar speaks Anthropic protocol but the
        // remote model is the abliterated Qwen NF4 build, which does
        // not support tools/vision/json-schema in B-37A's bring-up.
        RuntimeCapabilities {
            supports_tools: false,
            supports_streaming: false,
            supports_vision: false,
            max_context: 32_000, // E-32: raised from 25K to 32K (native Qwen2.5-Coder-7B window)
            max_output: self.max_tokens,
            json_schema_mode: false,
            supports_prefill: false,
            supports_caching: false,
            native_thinking_budget: false,
        }
    }

    fn health_check(&self) -> RuntimeHealthFuture<'_> {
        let url = self.endpoint("/health");
        let client = self.health_http.clone();
        Box::pin(async move {
            match client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => RuntimeHealth::Ready,
                Ok(resp) => unavailable(format!(
                    "sidecar /health returned HTTP {}",
                    resp.status().as_u16()
                )),
                Err(err) => unavailable(format!("sidecar /health unreachable: {err}")),
            }
        })
    }

    fn complete(
        &self,
        messages: Vec<Message>,
        config: CompletionConfig,
    ) -> RuntimeFuture<'_, CompletionResponse> {
        let request = self.build_request(messages, &config);
        let url = self.endpoint("/v1/messages");
        let client = self.http.clone();
        Box::pin(async move {
            let response = client
                .post(&url)
                .header("content-type", "application/json")
                .json(&request)
                .send()
                .await
                .map_err(|e| RuntimeAdapterError::Network(e.to_string()))?;

            if !response.status().is_success() {
                let status = response.status().as_u16();
                let body = response.text().await.unwrap_or_default();
                return Err(RuntimeAdapterError::Other(format!(
                    "sidecar /v1/messages returned HTTP {status}: {body}"
                )));
            }

            // Try the strict MessageResponse path first. Some sidecar
            // builds return a slightly different shape — fall back to
            // a generic JSON deserialise + best-effort text extraction
            // so this adapter doesn't break on minor protocol drift.
            let raw = response
                .bytes()
                .await
                .map_err(|e| RuntimeAdapterError::Network(e.to_string()))?;

            match serde_json::from_slice::<MessageResponse>(&raw) {
                Ok(parsed) => {
                    let mut text = String::new();
                    for block in &parsed.content {
                        if let crate::types::OutputContentBlock::Text { text: t } = block {
                            text.push_str(t);
                        }
                    }
                    Ok(CompletionResponse {
                        text,
                        stop_reason: parsed.stop_reason.clone(),
                        usage: (parsed.usage.input_tokens, parsed.usage.output_tokens),
                        model: parsed.model,
                    })
                }
                Err(_) => {
                    let value: Value = serde_json::from_slice(&raw).map_err(|e| {
                        RuntimeAdapterError::Other(format!(
                            "sidecar returned unparseable response: {e}"
                        ))
                    })?;
                    let text = extract_text(&value).unwrap_or_default();
                    Ok(CompletionResponse {
                        text,
                        stop_reason: None,
                        usage: (0, 0),
                        model: request.model.clone(),
                    })
                }
            }
        })
    }

    fn warm(&self) -> RuntimeFuture<'_, ()> {
        // E-3 sub-phase 4B — pre-warm the sidecar. A HEAD request to
        // /health is the cheapest probe that exercises the HTTP path
        // (TLS handshake / keep-alive setup) without sending tokens
        // through the model. The 1-token CUDA-graph warmup envisioned
        // by arXiv:2604.06664 Foundry is reserved for follow-up work —
        // /health hitting the FastAPI router is enough to keep the
        // connection pool warm and surface a Ready/Unavailable signal
        // through the same tracing channel as the SSB writer in 4B.
        let url = self.endpoint("/health");
        let client = self.health_http.clone();
        Box::pin(async move {
            // 1s timeout for warm: we already accept failure silently
            // via the tracing::warn line, so a long hang would only
            // delay subsequent scheduling decisions. Falls back to the
            // adapter's default health_http timeout (5s) when reqwest
            // does not honour the per-call timeout (it does on every
            // currently-supported transport).
            let started = OsClock.instant_now();
            let response = client
                .head(&url)
                .timeout(std::time::Duration::from_secs(1))
                .send()
                .await
                .map_err(|e| RuntimeAdapterError::Network(e.to_string()))?;
            if !response.status().is_success() {
                // Sidecars built before the HEAD route landed return
                // 405 Method Not Allowed; fall back to GET so the warm
                // path stays compatible with the abliterated server.py.
                if response.status().as_u16() == 405 {
                    let response = client
                        .get(&url)
                        .timeout(std::time::Duration::from_secs(1))
                        .send()
                        .await
                        .map_err(|e| RuntimeAdapterError::Network(e.to_string()))?;
                    if !response.status().is_success() {
                        return Err(RuntimeAdapterError::Other(format!(
                            "sidecar /health (GET fallback) returned HTTP {}",
                            response.status().as_u16()
                        )));
                    }
                } else {
                    return Err(RuntimeAdapterError::Other(format!(
                        "sidecar /health (HEAD) returned HTTP {}",
                        response.status().as_u16()
                    )));
                }
            }
            tracing::debug!(
                target = "runtime_adapter_sidecar",
                e3_runtime_handoff_event = "runtime_prewarm_complete",
                backend = "sidecar",
                duration_ms = started.elapsed().as_millis() as u64,
                "sidecar warm() ping ok"
            );
            Ok(())
        })
    }

    fn stream(
        &self,
        messages: Vec<Message>,
        config: CompletionConfig,
    ) -> BoxStream<'static, Result<StreamChunk, RuntimeAdapterError>> {
        // B-37 WARN-5 fix — real SSE consumer.
        //
        // The sidecar speaks the Anthropic `/v1/messages` protocol with
        // `stream: true` (per CLAUDE.md "Remote sidecar (canonical)" and
        // `server.py` `MessageRequest.stream`). We POST the same envelope
        // used by `complete()` with `stream=true` flipped on, then parse
        // the SSE response line-by-line into `StreamChunk` variants. No
        // new dependencies: `reqwest::Response::bytes_stream()` +
        // `futures_util::stream::StreamExt::unfold` is enough.
        let mut request = self.build_request(messages, &config);
        request.stream = true;
        let url = self.endpoint("/v1/messages");
        let client = self.http.clone();
        let state = SidecarStreamState::Connecting {
            client,
            url,
            request: Box::new(request),
        };
        Box::pin(stream::unfold(state, |state| async move {
            match state {
                SidecarStreamState::Connecting {
                    client,
                    url,
                    request,
                } => match client
                    .post(&url)
                    .header("content-type", "application/json")
                    .json(&request)
                    .send()
                    .await
                {
                    Ok(resp) if resp.status().is_success() => Some((
                        Ok(StreamChunk::Warning {
                            message: "sidecar stream connected".to_string(),
                        }),
                        SidecarStreamState::Streaming {
                            response: resp,
                            buffer: String::new(),
                            done: false,
                        },
                    )),
                    Ok(resp) => {
                        let status = resp.status().as_u16();
                        let body = resp.text().await.unwrap_or_default();
                        Some((
                            Err(RuntimeAdapterError::Other(format!(
                                "sidecar /v1/messages returned HTTP {status}: {body}"
                            ))),
                            SidecarStreamState::Terminated,
                        ))
                    }
                    Err(e) => Some((
                        Err(RuntimeAdapterError::Network(e.to_string())),
                        SidecarStreamState::Terminated,
                    )),
                },
                SidecarStreamState::Streaming {
                    mut response,
                    mut buffer,
                    done,
                } => {
                    if done {
                        return None;
                    }
                    // Pull next bytes chunk; if a complete SSE event is
                    // already buffered, emit it.
                    loop {
                        if let Some((chunk, rest, terminal)) = pop_sse_event(&buffer) {
                            buffer = rest;
                            let new_done = terminal
                                || matches!(chunk, Ok(StreamChunk::Stop { .. }));
                            return Some((
                                chunk,
                                SidecarStreamState::Streaming {
                                    response,
                                    buffer,
                                    done: new_done,
                                },
                            ));
                        }
                        match response.chunk().await {
                            Ok(Some(bytes)) => {
                                match std::str::from_utf8(&bytes) {
                                    Ok(s) => buffer.push_str(s),
                                    Err(_) => buffer
                                        .push_str(&String::from_utf8_lossy(&bytes)),
                                }
                                continue;
                            }
                            Ok(None) => {
                                // Underlying stream closed. Emit Stop if
                                // the buffer is empty, else drain whatever
                                // remains as a final pass.
                                if buffer.trim().is_empty() {
                                    return Some((
                                        Ok(StreamChunk::Stop { reason: None }),
                                        SidecarStreamState::Terminated,
                                    ));
                                }
                                if let Some((chunk, rest, _terminal)) =
                                    pop_sse_event(&buffer)
                                {
                                    buffer = rest;
                                    return Some((
                                        chunk,
                                        SidecarStreamState::Streaming {
                                            response,
                                            buffer,
                                            done: true,
                                        },
                                    ));
                                }
                                return Some((
                                    Ok(StreamChunk::Stop { reason: None }),
                                    SidecarStreamState::Terminated,
                                ));
                            }
                            Err(e) => {
                                return Some((
                                    Err(RuntimeAdapterError::Network(e.to_string())),
                                    SidecarStreamState::Terminated,
                                ));
                            }
                        }
                    }
                }
                SidecarStreamState::Terminated => None,
            }
        })
        .filter(|item| {
            // Suppress the synthetic "stream connected" warning so callers
            // only see real deltas + Stop. Errors and other warnings pass
            // through untouched.
            let keep = !matches!(
                item,
                Ok(StreamChunk::Warning { message }) if message == "sidecar stream connected"
            );
            async move { keep }
        }))
    }
}

/// Internal state machine for SSE streaming. Held by
/// `futures_util::stream::unfold`.
enum SidecarStreamState {
    Connecting {
        client: reqwest::Client,
        url: String,
        request: Box<MessageRequest>,
    },
    Streaming {
        response: Response,
        buffer: String,
        done: bool,
    },
    Terminated,
}

/// Parse one SSE event out of `buffer` if a complete one is present.
/// Returns `(parsed_chunk, remaining_buffer, terminal_flag)`. SSE events
/// are separated by a blank line (`\n\n`). The function is allocation-
/// conservative: only when an event is extracted do we allocate the
/// returned `String`.
///
/// Anthropic event-stream protocol (sidecar mirrors this — CLAUDE.md
/// "Remote sidecar (canonical)" + server.py /v1/messages):
///   `event: <type>\ndata: <json>\n\n`
///
/// Recognised event types (subset):
///   - `message_start`               -> Warning (sentinel)
///   - `content_block_delta`         -> TextDelta (from delta.text)
///   - `message_delta`               -> Warning (carries stop_reason on close)
///   - `message_stop`                -> Stop { reason: None }
///   - everything else               -> Warning
fn pop_sse_event(buffer: &str) -> Option<(Result<StreamChunk, RuntimeAdapterError>, String, bool)> {
    let separator = "\n\n";
    let pos = buffer.find(separator)?;
    let raw_event = &buffer[..pos];
    let remaining = buffer[pos + separator.len()..].to_string();

    let mut event_name: Option<&str> = None;
    let mut data_line: Option<&str> = None;
    for line in raw_event.lines() {
        if let Some(rest) = line.strip_prefix("event:") {
            event_name = Some(rest.trim());
        } else if let Some(rest) = line.strip_prefix("data:") {
            data_line = Some(rest.trim());
        }
    }

    let event_name = event_name.unwrap_or("");
    let data_json: Value = match data_line {
        Some(payload) => match serde_json::from_str(payload) {
            Ok(v) => v,
            Err(_) => Value::Null,
        },
        None => Value::Null,
    };

    let chunk = match event_name {
        "message_start" => Ok(StreamChunk::Warning {
            message: "message_start".to_string(),
        }),
        "content_block_delta" => {
            let text = data_json
                .get("delta")
                .and_then(|d| d.get("text"))
                .and_then(|t| t.as_str())
                .map(|s| s.to_string());
            match text {
                Some(t) => Ok(StreamChunk::TextDelta { text: t }),
                None => Ok(StreamChunk::Warning {
                    message: "content_block_delta without text".to_string(),
                }),
            }
        }
        "message_delta" => {
            let stop_reason = data_json
                .get("delta")
                .and_then(|d| d.get("stop_reason"))
                .and_then(|s| s.as_str())
                .map(|s| s.to_string());
            if stop_reason.is_some() {
                Ok(StreamChunk::Stop {
                    reason: stop_reason,
                })
            } else {
                Ok(StreamChunk::Warning {
                    message: "message_delta".to_string(),
                })
            }
        }
        "message_stop" => Ok(StreamChunk::Stop { reason: None }),
        "ping" | "" => Ok(StreamChunk::Warning {
            message: "ping".to_string(),
        }),
        other => Ok(StreamChunk::Warning {
            message: format!("sidecar event: {other}"),
        }),
    };

    let terminal = matches!(chunk, Ok(StreamChunk::Stop { .. }));
    Some((chunk, remaining, terminal))
}

/// Best-effort text extraction from arbitrary JSON. Searches for a
/// `content[].text` shape (Anthropic), then `choices[].text` (OpenAI).
fn extract_text(value: &Value) -> Option<String> {
    if let Some(content) = value.get("content").and_then(|v| v.as_array()) {
        let mut buf = String::new();
        for block in content {
            if let Some(t) = block.get("text").and_then(|v| v.as_str()) {
                buf.push_str(t);
            }
        }
        if !buf.is_empty() {
            return Some(buf);
        }
    }
    if let Some(choices) = value.get("choices").and_then(|v| v.as_array()) {
        let mut buf = String::new();
        for choice in choices {
            if let Some(t) = choice.get("text").and_then(|v| v.as_str()) {
                buf.push_str(t);
            }
        }
        if !buf.is_empty() {
            return Some(buf);
        }
    }
    None
}

// Suppress unused-field warnings for the fields we read only inside
// `Self::from_factory_config` / `build_request` (some are stored for
// future sub-cycles to consume).
impl SidecarRuntime {
    #[must_use]
    pub fn server_url(&self) -> &str {
        &self.server_url
    }

    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    #[must_use]
    pub fn request_timeout(&self) -> Duration {
        self.request_timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Replay a single content_block_delta event through the SSE
    /// parser and assert TextDelta extraction.
    #[test]
    fn pop_sse_event_parses_content_block_delta() {
        let buf = "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"Hello\"}}\n\n";
        let (chunk, rest, terminal) = pop_sse_event(buf).expect("event present");
        assert!(rest.is_empty());
        assert!(!terminal);
        match chunk {
            Ok(StreamChunk::TextDelta { text }) => assert_eq!(text, "Hello"),
            other => panic!("expected TextDelta, got {other:?}"),
        }
    }

    /// message_stop must yield a terminal Stop chunk.
    #[test]
    fn pop_sse_event_parses_message_stop() {
        let buf = "event: message_stop\ndata: {}\n\n";
        let (chunk, _rest, terminal) = pop_sse_event(buf).expect("event present");
        assert!(terminal);
        match chunk {
            Ok(StreamChunk::Stop { reason: None }) => {}
            other => panic!("expected Stop, got {other:?}"),
        }
    }

    /// message_delta with a stop_reason payload terminates the stream.
    #[test]
    fn pop_sse_event_parses_message_delta_with_stop_reason() {
        let buf = "event: message_delta\ndata: {\"delta\":{\"stop_reason\":\"end_turn\"}}\n\n";
        let (chunk, _rest, terminal) = pop_sse_event(buf).expect("event present");
        assert!(terminal);
        match chunk {
            Ok(StreamChunk::Stop { reason: Some(r) }) => assert_eq!(r, "end_turn"),
            other => panic!("expected Stop(end_turn), got {other:?}"),
        }
    }

    /// Incomplete event (no double-newline) leaves the buffer untouched.
    #[test]
    fn pop_sse_event_waits_for_double_newline() {
        let buf = "event: content_block_delta\ndata: {\"delta\":{\"text\":\"hi\"}}\n";
        assert!(pop_sse_event(buf).is_none());
    }

    /// Multiple events in one buffer are parsed sequentially.
    #[test]
    fn pop_sse_event_handles_multiple_events() {
        let buf = "event: message_start\ndata: {}\n\nevent: content_block_delta\ndata: {\"delta\":{\"text\":\"abc\"}}\n\n";
        let (first, rest, _) = pop_sse_event(buf).expect("first event");
        match first {
            Ok(StreamChunk::Warning { message }) => assert_eq!(message, "message_start"),
            other => panic!("expected Warning, got {other:?}"),
        }
        let (second, rest2, _) = pop_sse_event(&rest).expect("second event");
        assert!(rest2.is_empty());
        match second {
            Ok(StreamChunk::TextDelta { text }) => assert_eq!(text, "abc"),
            other => panic!("expected TextDelta, got {other:?}"),
        }
    }

    /// Ping events are emitted as a Warning and do not terminate.
    #[test]
    fn pop_sse_event_passes_through_ping() {
        let buf = "event: ping\ndata: {}\n\n";
        let (chunk, _rest, terminal) = pop_sse_event(buf).expect("event present");
        assert!(!terminal);
        match chunk {
            Ok(StreamChunk::Warning { message }) => assert_eq!(message, "ping"),
            other => panic!("expected Warning(ping), got {other:?}"),
        }
    }
}
