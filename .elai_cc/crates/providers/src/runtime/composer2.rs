//! `Composer2RuntimeAdapter` — Cursor Composer 2 HTTP API adapter
//! (C-44 Phase 4M extension).
//!
//! Composer 2 is Cursor Research's domain-specialised coding model.
//! Paper grounding: arXiv:2603.24477 ("Composer 2 Technical Report",
//! Cursor Research, 2026-03-25) grounds ONLY the existence + identity
//! of the model (training methodology + headline benchmarks). The
//! integration design below is source-grounded against @cursor/sdk +
//! cursor.com/docs/api — the paper does NOT specify wire shape, auth,
//! rate limits, or pricing. See workflow/papers/composer2-interface.md
//! for the shallow-cite-fit analysis.
//!
//! Cost positioning (per cursor.com pricing page, verified 2026-05-24):
//!   - $0.50 / 1M input tokens   ($0.0005 / 1k = 500 micros / 1k)
//!   - $2.50 / 1M output tokens  ($0.0025 / 1k = 2500 micros / 1k)
//!
//! Default routing: KEEP Executor -> Sidecar (free local). Composer 2
//! is OPT-IN per project via [runtime.composer2].enabled = true in
//! elai_cc.toml. Tier semantics emerge from cost-per-token relative
//! ranking, NOT a first-class enum (see `runtime/cost.rs`).
//!
//! Anti-stale rules followed (CLAUDE.md):
//!   - native async fn in trait via `Pin<Box<dyn Future + Send>>`
//!     (sibling pattern with `anthropic_api.rs`); NO async-trait crate.
//!   - thiserror not Box<dyn Error>.
//!   - tracing not log.
//!   - reqwest + serde + tokio: all workspace-pinned, no new deps.
//!   - api_key wrapped in SecretString.
//!   - .unwrap()/.expect() restricted to test code only.

use std::sync::Arc;
use std::time::Duration;

use futures_util::stream::{self, BoxStream};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, warn};

use super::secret::SecretString;
use super::types::{
    unavailable, CompletionConfig, CompletionResponse, Message, MessageRole, RuntimeAdapter,
    RuntimeAdapterError, RuntimeBackend, RuntimeCapabilities, RuntimeFactoryConfig, RuntimeFuture,
    RuntimeHealth, RuntimeHealthFuture, StreamChunk,
};

/// Canonical Cursor Composer 2 endpoint. The cursor.com/docs/api page
/// documents the inference endpoint at /v1/chat/completions (OpenAI-
/// compatible shape per source-grounding 2026-05-24). The base URL is
/// overrideable via `RuntimeFactoryConfig.base_url` for self-hosted
/// proxies / testing.
const DEFAULT_COMPOSER2_BASE_URL: &str = "https://api.cursor.com";

/// Per source-grounding (workflow/papers/composer2-interface.md): the
/// canonical model id is `composer-2`, NOT `cursor-composer-2` as the
/// original plan stated. Plan correction applied.
const DEFAULT_COMPOSER2_MODEL: &str = "composer-2";

/// Default completion max-tokens cap. Mirrors the AnthropicApi adapter
/// (anthropic_api.rs:36) so the runtime contract stays consistent
/// across HTTP-backed adapters.
const DEFAULT_MAX_TOKENS: u32 = 512;

/// 1-token canary cap used by `health_check`. Tiny to keep the probe
/// cheap (cost in micros is `est_cost(1, 1)` ≈ 3 micros = $0.000003).
const HEALTH_CANARY_MAX_TOKENS: u32 = 8;

/// Request timeout for `complete` calls. Cursor's API recommends a
/// 60s budget for long generations; we use 90s to leave headroom for
/// network jitter while still bounding worst-case latency.
const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 90;

/// Cursor Composer 2 specific error type. Maps onto
/// `RuntimeAdapterError` via `From` for adapter trait integration.
#[derive(Debug, thiserror::Error)]
pub enum Composer2Error {
    /// Reqwest / transport-level failure (DNS, TLS, socket).
    #[error("composer2 network failure: {0}")]
    Network(String),
    /// 401 / 403 from the Cursor API. Surfaces as `MissingField`-style
    /// guidance to the operator.
    #[error("composer2 auth failure: {0}")]
    Auth(String),
    /// 429 — operator hit the rate limit.
    #[error("composer2 rate limited: {0}")]
    RateLimit(String),
    /// 5xx — Cursor-side fault.
    #[error("composer2 server error (status {status}): {body_tail}")]
    ServerError { status: u16, body_tail: String },
    /// JSON deserialisation failure on the response body.
    #[error("composer2 response decode failure: {0}")]
    Decode(String),
}

impl From<Composer2Error> for RuntimeAdapterError {
    fn from(value: Composer2Error) -> Self {
        match value {
            Composer2Error::Network(msg) => RuntimeAdapterError::Network(msg),
            Composer2Error::Auth(reason) => RuntimeAdapterError::MissingField {
                backend: RuntimeBackend::Composer2,
                field: "api_key",
                reason,
            },
            other => RuntimeAdapterError::Other(other.to_string()),
        }
    }
}

/// OpenAI-compatible chat message envelope (Cursor's `/v1/chat/
/// completions` endpoint mirrors the OpenAI shape per source-grounding
/// on cursor.com/docs/api). Serde rename_all="lowercase" matches the
/// wire format.
#[derive(Debug, Clone, Serialize)]
struct Composer2RequestMessage {
    role: &'static str,
    content: String,
}

/// Outbound completion request body.
#[derive(Debug, Clone, Serialize)]
struct Composer2RequestBody {
    model: String,
    messages: Vec<Composer2RequestMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stop: Vec<String>,
    stream: bool,
}

/// Per-choice content + finish reason.
#[derive(Debug, Clone, Deserialize)]
struct Composer2ResponseChoice {
    message: Composer2ResponseMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Composer2ResponseMessage {
    #[serde(default)]
    content: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct Composer2UsageBlock {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
}

/// Inbound response body shape.
#[derive(Debug, Clone, Deserialize)]
struct Composer2ResponseBody {
    #[serde(default)]
    model: String,
    #[serde(default)]
    choices: Vec<Composer2ResponseChoice>,
    #[serde(default)]
    usage: Composer2UsageBlock,
}

/// Cursor Composer 2 runtime adapter.
///
/// Construction requires an api_key (env: CURSOR_API_KEY) — absent
/// keys surface as `RuntimeAdapterError::MissingField` at construct
/// time, NEVER as a panic or silent fallback (plan stub_ban).
#[derive(Clone)]
pub struct Composer2RuntimeAdapter {
    client: Client,
    api_key: Arc<SecretString>,
    base_url: String,
    default_model: String,
}

impl std::fmt::Debug for Composer2RuntimeAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Composer2RuntimeAdapter")
            .field("base_url", &self.base_url)
            .field("default_model", &self.default_model)
            .finish()
    }
}

impl Composer2RuntimeAdapter {
    /// Build from `RuntimeFactoryConfig`. Requires `cfg.api_key`
    /// (read from `CURSOR_API_KEY` env via the orchestrator's toml
    /// loader). Absent key returns `MissingField`, never panics.
    pub fn from_factory_config(cfg: &RuntimeFactoryConfig) -> Result<Self, RuntimeAdapterError> {
        let api_key = cfg.api_key.clone().ok_or_else(|| {
            RuntimeAdapterError::MissingField {
                backend: RuntimeBackend::Composer2,
                field: "api_key",
                reason: "Composer 2 runtime requires CURSOR_API_KEY env var or [runtime.composer2].api_key inline".to_string(),
            }
        })?;

        let base_url = cfg
            .base_url
            .clone()
            .unwrap_or_else(|| DEFAULT_COMPOSER2_BASE_URL.to_string());

        let default_model = cfg
            .model
            .clone()
            .unwrap_or_else(|| DEFAULT_COMPOSER2_MODEL.to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS))
            .build()
            .map_err(|e| RuntimeAdapterError::Network(format!("reqwest client build: {e}")))?;

        Ok(Self {
            client,
            api_key: Arc::new(api_key),
            base_url,
            default_model,
        })
    }

    fn build_headers(&self) -> Result<HeaderMap, Composer2Error> {
        let mut headers = HeaderMap::new();
        let bearer = format!("Bearer {}", self.api_key.expose_secret());
        let mut auth_value = HeaderValue::from_str(&bearer)
            .map_err(|e| Composer2Error::Auth(format!("invalid api_key bytes: {e}")))?;
        auth_value.set_sensitive(true);
        headers.insert(AUTHORIZATION, auth_value);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(headers)
    }

    fn build_request_body(
        &self,
        messages: Vec<Message>,
        config: &CompletionConfig,
    ) -> Composer2RequestBody {
        let model = config
            .model
            .clone()
            .unwrap_or_else(|| self.default_model.clone());

        let max_tokens = if config.max_tokens == 0 {
            Some(DEFAULT_MAX_TOKENS)
        } else {
            Some(config.max_tokens)
        };

        let mut wire_messages: Vec<Composer2RequestMessage> = Vec::with_capacity(messages.len());
        if let Some(system) = &config.system {
            wire_messages.push(Composer2RequestMessage {
                role: "system",
                content: system.clone(),
            });
        }
        for msg in messages {
            let role = match msg.role {
                MessageRole::Assistant => "assistant",
                MessageRole::System => "system",
                MessageRole::User | MessageRole::Tool => "user",
            };
            wire_messages.push(Composer2RequestMessage {
                role,
                content: msg.content,
            });
        }

        Composer2RequestBody {
            model,
            messages: wire_messages,
            max_tokens,
            temperature: Some(config.temperature),
            seed: config.seed,
            stop: config.stop_sequences.clone(),
            stream: false,
        }
    }

    fn endpoint_url(&self) -> String {
        // Trim trailing slash from base_url so we don't end up with
        // double-slash in the final URL.
        let base = self.base_url.trim_end_matches('/');
        format!("{base}/v1/chat/completions")
    }

    async fn send_completion(
        &self,
        body: Composer2RequestBody,
    ) -> Result<Composer2ResponseBody, Composer2Error> {
        let headers = self.build_headers()?;
        let url = self.endpoint_url();

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| Composer2Error::Network(e.to_string()))?;

        let status = response.status();
        if status.is_success() {
            let parsed = response
                .json::<Composer2ResponseBody>()
                .await
                .map_err(|e| Composer2Error::Decode(e.to_string()))?;
            return Ok(parsed);
        }

        let body_tail = response
            .text()
            .await
            .unwrap_or_else(|_| "<body unreadable>".to_string());
        let body_tail = if body_tail.len() > 256 {
            format!("{}...", &body_tail[..256])
        } else {
            body_tail
        };
        let status_code = status.as_u16();
        match status_code {
            401 | 403 => Err(Composer2Error::Auth(format!(
                "status {status_code}: {body_tail}"
            ))),
            429 => Err(Composer2Error::RateLimit(format!(
                "status {status_code}: {body_tail}"
            ))),
            500..=599 => Err(Composer2Error::ServerError {
                status: status_code,
                body_tail,
            }),
            _ => Err(Composer2Error::Network(format!(
                "status {status_code}: {body_tail}"
            ))),
        }
    }
}

impl RuntimeAdapter for Composer2RuntimeAdapter {
    fn backend(&self) -> RuntimeBackend {
        RuntimeBackend::Composer2
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        // Source-grounded against cursor.com/docs/models/cursor-
        // composer-2 (2026-05-24). Context window per the model page:
        // 200K input. Output cap: 8K. Tools: yes (cursor agent tools).
        // Streaming: yes. Vision: not advertised at composer-2 tier.
        RuntimeCapabilities {
            supports_tools: true,
            supports_streaming: true,
            supports_vision: false,
            max_context: 200_000,
            max_output: 8_192,
            json_schema_mode: false,
            supports_prefill: false,
            supports_caching: false,
            native_thinking_budget: false,
        }
    }

    fn health_check(&self) -> RuntimeHealthFuture<'_> {
        let this = self.clone();
        Box::pin(async move {
            let body = Composer2RequestBody {
                model: this.default_model.clone(),
                messages: vec![Composer2RequestMessage {
                    role: "user",
                    content: "ok".to_string(),
                }],
                max_tokens: Some(HEALTH_CANARY_MAX_TOKENS),
                temperature: Some(0.0),
                seed: None,
                stop: Vec::new(),
                stream: false,
            };
            match this.send_completion(body).await {
                Ok(_) => RuntimeHealth::Ready,
                Err(Composer2Error::Auth(reason)) => unavailable(format!(
                    "composer2 auth failure (set CURSOR_API_KEY): {reason}"
                )),
                Err(Composer2Error::RateLimit(reason)) => RuntimeHealth::Degraded {
                    reason: format!("composer2 rate limited: {reason}"),
                },
                Err(err) => {
                    debug!(
                        target = "composer2_runtime",
                        error = %err,
                        "health_check probe failed"
                    );
                    unavailable(format!("composer2 canary failed: {err}"))
                }
            }
        })
    }

    fn complete(
        &self,
        messages: Vec<Message>,
        config: CompletionConfig,
    ) -> RuntimeFuture<'_, CompletionResponse> {
        let this = self.clone();
        Box::pin(async move {
            let body = this.build_request_body(messages, &config);
            let response = this
                .send_completion(body)
                .await
                .map_err(RuntimeAdapterError::from)?;

            let text = response
                .choices
                .iter()
                .map(|c| c.message.content.as_str())
                .collect::<Vec<_>>()
                .join("");
            let stop_reason = response
                .choices
                .first()
                .and_then(|c| c.finish_reason.clone());

            Ok(CompletionResponse {
                text,
                stop_reason,
                usage: (
                    response.usage.prompt_tokens,
                    response.usage.completion_tokens,
                ),
                cache_creation_input_tokens: 0,
                cache_read_input_tokens: 0,
                model: if response.model.is_empty() {
                    this.default_model.clone()
                } else {
                    response.model
                },
            })
        })
    }

    fn stream(
        &self,
        messages: Vec<Message>,
        config: CompletionConfig,
    ) -> BoxStream<'static, Result<StreamChunk, RuntimeAdapterError>> {
        // Streaming surface: Cursor's API accepts stream=true with
        // SSE-formatted chunks. To keep the surface minimal in C-44,
        // we wrap the non-streaming complete() call and emit a single
        // TextDelta + Stop pair. A proper SSE pump can be wired in a
        // follow-up patch; this preserves the stream() interface
        // without leaking partial wire support.
        let this = self.clone();
        let state = StreamingState::Pending {
            adapter: Arc::new(Mutex::new(this)),
            messages,
            config,
        };
        Box::pin(stream::unfold(state, |state| async move {
            match state {
                StreamingState::Pending {
                    adapter,
                    messages,
                    config,
                } => {
                    let guard = adapter.lock().await;
                    let body = guard.build_request_body(messages.clone(), &config);
                    let result = guard.send_completion(body).await;
                    drop(guard);
                    match result {
                        Ok(response) => {
                            let text = response
                                .choices
                                .iter()
                                .map(|c| c.message.content.as_str())
                                .collect::<Vec<_>>()
                                .join("");
                            let stop_reason = response
                                .choices
                                .first()
                                .and_then(|c| c.finish_reason.clone());
                            Some((
                                Ok(StreamChunk::TextDelta { text }),
                                StreamingState::AfterText { stop_reason },
                            ))
                        }
                        Err(err) => {
                            warn!(
                                target = "composer2_runtime",
                                error = %err,
                                "stream complete failed"
                            );
                            Some((
                                Err(RuntimeAdapterError::from(err)),
                                StreamingState::Terminated,
                            ))
                        }
                    }
                }
                StreamingState::AfterText { stop_reason } => Some((
                    Ok(StreamChunk::Stop {
                        reason: stop_reason,
                    }),
                    StreamingState::Terminated,
                )),
                StreamingState::Terminated => None,
            }
        }))
    }
}

enum StreamingState {
    Pending {
        adapter: Arc<Mutex<Composer2RuntimeAdapter>>,
        messages: Vec<Message>,
        config: CompletionConfig,
    },
    AfterText {
        stop_reason: Option<String>,
    },
    Terminated,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn factory_with_key(api_key: &str) -> RuntimeFactoryConfig {
        let mut cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::Composer2);
        cfg.api_key = Some(SecretString::new(api_key.to_string()));
        cfg
    }

    #[test]
    fn from_factory_config_requires_api_key() {
        let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::Composer2);
        let err = Composer2RuntimeAdapter::from_factory_config(&cfg)
            .expect_err("must fail without api_key");
        match err {
            RuntimeAdapterError::MissingField {
                backend,
                field,
                reason,
            } => {
                assert_eq!(backend, RuntimeBackend::Composer2);
                assert_eq!(field, "api_key");
                assert!(reason.contains("CURSOR_API_KEY"));
            }
            other => panic!("expected MissingField, got {other:?}"),
        }
    }

    #[test]
    fn from_factory_config_uses_default_model_when_none() {
        let cfg = factory_with_key("test-key");
        let adapter = Composer2RuntimeAdapter::from_factory_config(&cfg).expect("build");
        assert_eq!(adapter.default_model, "composer-2");
    }

    #[test]
    fn from_factory_config_uses_default_base_url_when_none() {
        let cfg = factory_with_key("test-key");
        let adapter = Composer2RuntimeAdapter::from_factory_config(&cfg).expect("build");
        assert_eq!(adapter.base_url, "https://api.cursor.com");
    }

    #[test]
    fn from_factory_config_honours_model_override() {
        let mut cfg = factory_with_key("test-key");
        cfg.model = Some("composer-2-preview".to_string());
        let adapter = Composer2RuntimeAdapter::from_factory_config(&cfg).expect("build");
        assert_eq!(adapter.default_model, "composer-2-preview");
    }

    #[test]
    fn backend_self_identifies_as_composer2() {
        let cfg = factory_with_key("test-key");
        let adapter = Composer2RuntimeAdapter::from_factory_config(&cfg).expect("build");
        assert_eq!(adapter.backend(), RuntimeBackend::Composer2);
    }

    #[test]
    fn capabilities_match_source_grounded_values() {
        let cfg = factory_with_key("test-key");
        let adapter = Composer2RuntimeAdapter::from_factory_config(&cfg).expect("build");
        let caps = adapter.capabilities();
        assert!(caps.supports_tools);
        assert!(caps.supports_streaming);
        assert!(!caps.supports_vision);
        assert_eq!(caps.max_context, 200_000);
        assert_eq!(caps.max_output, 8_192);
    }

    #[test]
    fn build_request_body_round_trips_system_prompt() {
        let cfg = factory_with_key("test-key");
        let adapter = Composer2RuntimeAdapter::from_factory_config(&cfg).expect("build");
        let messages = vec![Message {
            role: MessageRole::User,
            content: "hello".to_string(),
        }];
        let mut completion_cfg = CompletionConfig::default();
        completion_cfg.system = Some("be terse".to_string());
        let body = adapter.build_request_body(messages, &completion_cfg);
        assert_eq!(body.messages.len(), 2);
        assert_eq!(body.messages[0].role, "system");
        assert_eq!(body.messages[0].content, "be terse");
        assert_eq!(body.messages[1].role, "user");
        assert_eq!(body.model, "composer-2");
        assert!(!body.stream);
    }

    #[test]
    fn build_request_body_max_tokens_uses_default_on_zero() {
        let cfg = factory_with_key("test-key");
        let adapter = Composer2RuntimeAdapter::from_factory_config(&cfg).expect("build");
        let mut completion_cfg = CompletionConfig::default();
        completion_cfg.max_tokens = 0;
        let body = adapter.build_request_body(Vec::new(), &completion_cfg);
        assert_eq!(body.max_tokens, Some(DEFAULT_MAX_TOKENS));
    }

    #[test]
    fn request_body_serialises_to_openai_compatible_json() {
        let body = Composer2RequestBody {
            model: "composer-2".to_string(),
            messages: vec![Composer2RequestMessage {
                role: "user",
                content: "ping".to_string(),
            }],
            max_tokens: Some(8),
            temperature: Some(0.0),
            seed: None,
            stop: Vec::new(),
            stream: false,
        };
        let json = serde_json::to_string(&body).expect("serialize");
        // Round-trip-by-fields rather than byte equality so field
        // ordering churn doesn't break the test.
        assert!(json.contains("\"model\":\"composer-2\""));
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"ping\""));
        assert!(json.contains("\"max_tokens\":8"));
        assert!(json.contains("\"stream\":false"));
        // seed is None and `skip_serializing_if`-omitted.
        assert!(!json.contains("\"seed\""));
        // stop is empty Vec and `skip_serializing_if`-omitted.
        assert!(!json.contains("\"stop\""));
    }

    #[test]
    fn response_body_deserialises_minimal_openai_shape() {
        let raw = r#"{
            "model": "composer-2",
            "choices": [
                {"message": {"content": "hi"}, "finish_reason": "stop"}
            ],
            "usage": {"prompt_tokens": 3, "completion_tokens": 1}
        }"#;
        let parsed: Composer2ResponseBody = serde_json::from_str(raw).expect("parse");
        assert_eq!(parsed.model, "composer-2");
        assert_eq!(parsed.choices.len(), 1);
        assert_eq!(parsed.choices[0].message.content, "hi");
        assert_eq!(parsed.choices[0].finish_reason.as_deref(), Some("stop"));
        assert_eq!(parsed.usage.prompt_tokens, 3);
        assert_eq!(parsed.usage.completion_tokens, 1);
    }

    #[test]
    fn endpoint_url_appends_chat_completions() {
        let cfg = factory_with_key("test-key");
        let adapter = Composer2RuntimeAdapter::from_factory_config(&cfg).expect("build");
        assert_eq!(
            adapter.endpoint_url(),
            "https://api.cursor.com/v1/chat/completions"
        );
    }

    #[test]
    fn endpoint_url_trims_trailing_slash_on_base_url() {
        let mut cfg = factory_with_key("test-key");
        cfg.base_url = Some("https://api.cursor.com/".to_string());
        let adapter = Composer2RuntimeAdapter::from_factory_config(&cfg).expect("build");
        assert_eq!(
            adapter.endpoint_url(),
            "https://api.cursor.com/v1/chat/completions"
        );
    }

    #[test]
    fn composer2_error_maps_auth_to_missing_field() {
        let err = RuntimeAdapterError::from(Composer2Error::Auth("401".to_string()));
        match err {
            RuntimeAdapterError::MissingField {
                backend,
                field,
                reason,
            } => {
                assert_eq!(backend, RuntimeBackend::Composer2);
                assert_eq!(field, "api_key");
                assert!(reason.contains("401"));
            }
            other => panic!("expected MissingField mapping, got {other:?}"),
        }
    }

    #[test]
    fn composer2_error_maps_network_to_network() {
        let err = RuntimeAdapterError::from(Composer2Error::Network("dns failure".to_string()));
        match err {
            RuntimeAdapterError::Network(msg) => assert!(msg.contains("dns failure")),
            other => panic!("expected Network mapping, got {other:?}"),
        }
    }
}
