//! `SglangRuntime` — E-26 native HTTP adapter for a SGLang server.
//!
//! Wire protocol (sgl-project/sglang docs v0.4+, verified 2026-05):
//! - Completion: `POST /v1/chat/completions` — OpenAI-compatible (same
//!   shape as vLLM's reference impl). XGrammar / Outlines / LMFE
//!   selection happens via `extra_body.guided_json` +
//!   `extra_body.guided_regex` + `extra_body.backend`. The Rust adapter
//!   forwards the canonical backend string and lets the SGLang server
//!   route to its compiled grammar engine.
//! - Stream: SSE `data:`-prefixed JSON; final `data: [DONE]`.
//! - Health: `GET /v1/models` — returns `{"data":[...]}`.
//!
//! Why SGLang is a separate adapter (NOT folded into `vllm.rs`):
//!   * SGLang's batch-aware `CustomLogitProcessor` plugin
//!     (sgl-project/sglang#2291) is closer to ELAI's per-request
//!     LogitsProcessor chain than vLLM V1's global batch-level
//!     processors (docs.vllm.ai/en/v0.8.1 v1_user_guide).
//!   * Default base URL differs (30000 vs vLLM's 8000) and the
//!     structured-output backend selector lives in `extra_body`, not
//!     `response_format`.
//!   * Cold-start + RadixAttention + XGrammar default (per
//!     blog.mlc.ai/2026/05/04/xgrammar-2-fast-customizable-structured-
//!     generation) make SGLang the chosen cloud-throughput tier per
//!     the source report `26-inference-engines-2026.md`.
//!
//! Disambiguation with existing `SglangConfig` in
//! `crates/orchestrator/src/media_validator/types.rs:364`: that struct
//! is media-validator scoped (request-local Tier-2 VLM path); this
//! `SglangRuntimeConfig` is RuntimeAdapter-scoped (long-lived
//! health-probed cloud throughput adapter). Plan body explicitly says
//! DO NOT collapse — different lifecycle.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::stream::{self, BoxStream};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::event_sink::RuntimeEvent;
use super::runtime_event_sink;
use super::types::{
    unavailable, CompletionConfig, CompletionResponse, Message, MessageRole, RuntimeAdapter,
    RuntimeAdapterError, RuntimeBackend, RuntimeCapabilities, RuntimeFactoryConfig, RuntimeFuture,
    RuntimeHealth, RuntimeHealthFuture, StreamChunk,
};

const BACKEND: RuntimeBackend = RuntimeBackend::Sglang;
const DEFAULT_BASE_URL: &str = "http://localhost:30000";
const DEFAULT_MODEL: &str = "Qwen/Qwen2.5-Coder-7B-Instruct";
const REQUEST_TIMEOUT_SECS: u64 = 120;
const HEALTH_TIMEOUT_SECS: u64 = 5;
const DEFAULT_MAX_TOKENS: u32 = 512;

/// Structured-output backend selector forwarded to SGLang via
/// `extra_body.backend`. Per docs.sglang.ai/backend/structured_outputs
/// the server-side resolver routes the request to XGrammar / Outlines /
/// LMFE. Default is XGrammar — matches blog.mlc.ai 2026-05-04 (XGrammar
/// is the default in SGLang, vLLM, TRT-LLM, MLC-LLM).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuredOutputBackend {
    XGrammar,
    Outlines,
    Lmfe,
}

impl Default for StructuredOutputBackend {
    fn default() -> Self {
        Self::XGrammar
    }
}

impl StructuredOutputBackend {
    /// Snake_case wire tag the SGLang server expects.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::XGrammar => "xgrammar",
            Self::Outlines => "outlines",
            Self::Lmfe => "lm-format-enforcer",
        }
    }
}

/// Allowed attention/attention-adjacent backends for SGLang det-mode.
/// Per docs.sglang.ai advanced_features (det-repos-S1, fetched 2026-06-11):
/// det-mode requires FlashInfer, FlashAttention 3, or Triton — other
/// backends (raw CUDA, ROCm) do not support the kernel swaps det-mode
/// relies on. Peer pattern: `StructuredOutputBackend` at line 61.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SglangDetModeBackend {
    FlashInfer,
    Fa3,
    Triton,
}

impl SglangDetModeBackend {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FlashInfer => "flashinfer",
            Self::Fa3 => "fa3",
            Self::Triton => "triton",
        }
    }

    /// True if a raw backend name string is an allowed det-mode backend.
    /// Case-sensitive, matches `as_str()` output.
    #[must_use]
    pub fn is_allowed_str(name: &str) -> bool {
        matches!(name, "flashinfer" | "fa3" | "triton")
    }
}

/// SGLang det-mode capability check request. Maps the three requirements
/// from J-13 plan item 4 / E-26 contract note:
/// - `enable_deterministic_inference`: the `--enable-deterministic-inference`
///   server flag was set.
/// - `sampling_seed`: the per-request seed (distinct from the process
///   ELAI_SEED pin; this is the request-level sampling seed J-13 threads
///   through `MessageRequest.seed`).
/// - `backend`: the raw backend name string the SGLang server reports
///   using for this request. Validated against `SglangDetModeBackend::is_allowed_str`.
#[derive(Debug, Clone, PartialEq)]
pub struct SglangDetModeRequest {
    pub enable_deterministic_inference: bool,
    pub sampling_seed: Option<u64>,
    pub backend: String,
}

/// Det-mode capability check verdict. Pure, no HTTP, no server probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SglangDetModeError {
    /// `--enable-deterministic-inference` was not set.
    MissingFlag,
    /// per-request sampling_seed was not provided.
    MissingSamplingSeed,
    /// backend is not in the FlashInfer/FA3/Triton allowed set.
    DisallowedBackend { backend: String },
}

/// Validate a SGLang det-mode request against the three requirements.
/// Returns `Ok(())` when all three are present and the backend is allowed;
/// returns the first failing `SglangDetModeError` otherwise. Order:
/// flag > seed > backend (flag is the cheapest gate; a request without
/// the flag does not need seed/backend validation).
// WIRE-DARK[J-13-S5-#8]
#[must_use]
pub fn validate_sglang_det_mode(req: &SglangDetModeRequest) -> Result<(), SglangDetModeError> {
    if !req.enable_deterministic_inference {
        return Err(SglangDetModeError::MissingFlag);
    }
    if req.sampling_seed.is_none() {
        return Err(SglangDetModeError::MissingSamplingSeed);
    }
    if !SglangDetModeBackend::is_allowed_str(&req.backend) {
        return Err(SglangDetModeError::DisallowedBackend {
            backend: req.backend.clone(),
        });
    }
    Ok(())
}

/// Long-lived SGLang HTTP runtime adapter config. Separate struct from
/// the media-validator `SglangConfig` (different lifecycle).
#[derive(Debug, Clone, PartialEq)]
pub struct SglangRuntimeConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub request_timeout: Duration,
    pub structured_output_backend: StructuredOutputBackend,
}

impl Default for SglangRuntimeConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            api_key: None,
            request_timeout: Duration::from_secs(REQUEST_TIMEOUT_SECS),
            structured_output_backend: StructuredOutputBackend::default(),
        }
    }
}

/// SGLang HTTP runtime adapter.
#[derive(Debug, Clone)]
pub struct SglangRuntime {
    base_url: String,
    model: String,
    http: reqwest::Client,
    health_http: reqwest::Client,
    structured_output_backend: StructuredOutputBackend,
    cold_start_seen: Arc<AtomicBool>,
}

impl SglangRuntime {
    #[must_use]
    pub fn new(cfg: RuntimeFactoryConfig) -> Self {
        let base_url = cfg
            .base_url
            .clone()
            .or_else(|| std::env::var("ELAI_SGLANG_BASE_URL").ok())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let model = cfg
            .model
            .clone()
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        let health_http = reqwest::Client::builder()
            .timeout(Duration::from_secs(HEALTH_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        let structured_output_backend = StructuredOutputBackend::default();
        runtime_event_sink().record(Self::registered_event(
            &base_url,
            structured_output_backend,
        ));
        Self {
            base_url,
            model,
            http,
            health_http,
            structured_output_backend,
            cold_start_seen: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Construct from an explicit [`SglangRuntimeConfig`]. Used by tests
    /// + advanced callers that want to override the structured-output
    /// backend selector without going through the env-driven factory.
    #[must_use]
    pub fn with_runtime_config(cfg: SglangRuntimeConfig, model: Option<String>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(cfg.request_timeout)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        let health_http = reqwest::Client::builder()
            .timeout(Duration::from_secs(HEALTH_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        let structured_output_backend = cfg.structured_output_backend;
        runtime_event_sink().record(Self::registered_event(
            &cfg.base_url,
            structured_output_backend,
        ));
        Self {
            base_url: cfg.base_url,
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            http,
            health_http,
            structured_output_backend,
            cold_start_seen: Arc::new(AtomicBool::new(false)),
        }
    }

    fn registered_event(base_url: &str, backend: StructuredOutputBackend) -> RuntimeEvent {
        RuntimeEvent::SglangBackendRegistered {
            base_url: base_url.to_string(),
            structured_output_backend: backend.as_str().to_string(),
        }
    }

    fn observe_cold_start(seen: &AtomicBool, elapsed_ms: u64) -> Option<RuntimeEvent> {
        match seen.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => Some(RuntimeEvent::RuntimeColdStartObserved {
                backend: RuntimeBackend::Sglang,
                elapsed_ms,
            }),
            Err(_) => None,
        }
    }

    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    #[must_use]
    pub fn structured_output_backend(&self) -> StructuredOutputBackend {
        self.structured_output_backend
    }

    #[must_use]
    pub fn canonical_name() -> &'static str {
        "sglang"
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }

    fn build_messages(messages: &[Message]) -> Vec<OpenAiMessage> {
        messages
            .iter()
            .map(|m| OpenAiMessage {
                role: match m.role {
                    MessageRole::Assistant => "assistant",
                    MessageRole::System => "system",
                    MessageRole::Tool => "tool",
                    MessageRole::User => "user",
                }
                .to_string(),
                content: m.content.clone(),
            })
            .collect()
    }
}

impl RuntimeAdapter for SglangRuntime {
    fn backend(&self) -> RuntimeBackend {
        BACKEND
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities {
            supports_tools: true,
            supports_streaming: true,
            supports_vision: false,
            // SGLang default Qwen2.5-Coder-7B context window per HF model card.
            max_context: 32_768,
            max_output: DEFAULT_MAX_TOKENS,
            // SGLang surfaces XGrammar / Outlines / LMFE through
            // `extra_body.backend`; json_schema mode is the headline feature.
            json_schema_mode: true,
            supports_prefill: false,
            supports_caching: true,
            native_thinking_budget: false,
        }
    }

    fn health_check(&self) -> RuntimeHealthFuture<'_> {
        // GET /v1/models — SGLang exposes the standard OpenAI surface.
        // 200 with a non-empty data array == Ready.
        let models_url = self.endpoint("/v1/models");
        let client = self.health_http.clone();
        let cold_start_seen = Arc::clone(&self.cold_start_seen);
        Box::pin(async move {
            let started = Instant::now();
            let health = match client.get(&models_url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<ModelsResponse>().await {
                        Ok(m) if !m.data.is_empty() => RuntimeHealth::Ready,
                        Ok(_) => unavailable("sglang /v1/models returned empty data array"),
                        Err(e) => unavailable(format!("sglang /v1/models parse failed: {e}")),
                    }
                }
                Ok(resp) => unavailable(format!(
                    "sglang /v1/models returned HTTP {}",
                    resp.status().as_u16()
                )),
                Err(err) => unavailable(format!("sglang unreachable: {err}")),
            };
            if matches!(health, RuntimeHealth::Ready) {
                let elapsed_ms = started.elapsed().as_millis() as u64;
                if let Some(event) = Self::observe_cold_start(&cold_start_seen, elapsed_ms) {
                    runtime_event_sink().record(event);
                }
            }
            health
        })
    }

    fn complete(
        &self,
        messages: Vec<Message>,
        config: CompletionConfig,
    ) -> RuntimeFuture<'_, CompletionResponse> {
        let url = self.endpoint("/v1/chat/completions");
        let client = self.http.clone();
        let default_model = self.model.clone();
        let backend_tag = self.structured_output_backend.as_str();
        Box::pin(async move {
            let model = config.model.clone().unwrap_or(default_model);
            let max_tokens = if config.max_tokens == 0 {
                DEFAULT_MAX_TOKENS
            } else {
                config.max_tokens
            };
            let body = OpenAiChatRequest {
                model: model.clone(),
                messages: Self::build_messages(&messages),
                max_tokens: Some(max_tokens),
                temperature: Some(config.temperature),
                stream: false,
                stop: if config.stop_sequences.is_empty() {
                    None
                } else {
                    Some(config.stop_sequences.clone())
                },
                seed: config.seed,
                // SGLang routes via extra_body.backend — XGrammar default,
                // Outlines + LMFE legacy. Forwarded as a top-level field that
                // SGLang's request handler reads alongside response_format.
                extra_body: Some(SglangExtraBody {
                    backend: backend_tag.to_string(),
                }),
            };

            let response = client
                .post(&url)
                .json(&body)
                .send()
                .await
                .map_err(|e| RuntimeAdapterError::Network(e.to_string()))?;

            if !response.status().is_success() {
                let code = response.status().as_u16();
                let body_text = response.text().await.unwrap_or_default();
                return Err(RuntimeAdapterError::Other(format!(
                    "sglang /v1/chat/completions returned HTTP {code}: {body_text}"
                )));
            }

            let parsed: OpenAiChatResponse = response
                .json()
                .await
                .map_err(|e| RuntimeAdapterError::Network(e.to_string()))?;
            let choice = parsed
                .choices
                .into_iter()
                .next()
                .ok_or_else(|| RuntimeAdapterError::Other("sglang returned no choices".into()))?;
            Ok(CompletionResponse {
                text: choice.message.content,
                stop_reason: choice.finish_reason,
                usage: parsed
                    .usage
                    .map(|u| (u.prompt_tokens, u.completion_tokens))
                    .unwrap_or((0, 0)),
                model,
            })
        })
    }

    fn stream(
        &self,
        messages: Vec<Message>,
        config: CompletionConfig,
    ) -> BoxStream<'static, Result<StreamChunk, RuntimeAdapterError>> {
        let url = self.endpoint("/v1/chat/completions");
        let client = self.http.clone();
        let default_model = self.model.clone();
        let backend_tag = self.structured_output_backend.as_str();
        let model = config.model.clone().unwrap_or(default_model);
        let max_tokens = if config.max_tokens == 0 {
            DEFAULT_MAX_TOKENS
        } else {
            config.max_tokens
        };
        let body = OpenAiChatRequest {
            model,
            messages: Self::build_messages(&messages),
            max_tokens: Some(max_tokens),
            temperature: Some(config.temperature),
            stream: true,
            stop: if config.stop_sequences.is_empty() {
                None
            } else {
                Some(config.stop_sequences.clone())
            },
            seed: config.seed,
            extra_body: Some(SglangExtraBody {
                backend: backend_tag.to_string(),
            }),
        };

        Box::pin(stream::unfold(
            SglangStreamState::Connecting {
                client,
                url,
                body: Box::new(body),
            },
            sglang_stream_step,
        ))
    }
}

enum SglangStreamState {
    Connecting {
        client: reqwest::Client,
        url: String,
        body: Box<OpenAiChatRequest>,
    },
    Streaming {
        buffer: String,
        response: reqwest::Response,
    },
    Done,
}

async fn sglang_stream_step(
    state: SglangStreamState,
) -> Option<(Result<StreamChunk, RuntimeAdapterError>, SglangStreamState)> {
    match state {
        SglangStreamState::Connecting { client, url, body } => {
            let response = match client.post(&url).json(&*body).send().await {
                Ok(r) => r,
                Err(e) => {
                    return Some((
                        Err(RuntimeAdapterError::Network(e.to_string())),
                        SglangStreamState::Done,
                    ));
                }
            };
            if !response.status().is_success() {
                let code = response.status().as_u16();
                let body_text = response.text().await.unwrap_or_default();
                return Some((
                    Err(RuntimeAdapterError::Other(format!(
                        "sglang stream HTTP {code}: {body_text}"
                    ))),
                    SglangStreamState::Done,
                ));
            }
            Some((
                Ok(StreamChunk::Warning {
                    message: "sglang stream connected".to_string(),
                }),
                SglangStreamState::Streaming {
                    buffer: String::new(),
                    response,
                },
            ))
        }
        SglangStreamState::Streaming {
            mut buffer,
            mut response,
        } => loop {
            if let Some(idx) = buffer.find('\n') {
                let raw_line: String = buffer.drain(..=idx).collect();
                let line = raw_line.trim_end_matches(['\r', '\n']).to_string();
                if line.is_empty() {
                    continue;
                }
                let payload = line.strip_prefix("data: ").unwrap_or(&line).to_string();
                if payload.trim() == "[DONE]" {
                    return Some((
                        Ok(StreamChunk::Stop { reason: None }),
                        SglangStreamState::Done,
                    ));
                }
                match serde_json::from_str::<Value>(&payload) {
                    Ok(value) => {
                        if let Some(chunk) = extract_openai_delta(&value) {
                            return Some((
                                Ok(chunk),
                                SglangStreamState::Streaming { buffer, response },
                            ));
                        }
                        continue;
                    }
                    Err(_) => {
                        return Some((
                            Ok(StreamChunk::Warning {
                                message: format!("non-json stream line: {payload}"),
                            }),
                            SglangStreamState::Streaming { buffer, response },
                        ));
                    }
                }
            }
            match response.chunk().await {
                Ok(Some(bytes)) => match std::str::from_utf8(&bytes) {
                    Ok(s) => buffer.push_str(s),
                    Err(_) => buffer.push_str(&String::from_utf8_lossy(&bytes)),
                },
                Ok(None) => {
                    return Some((
                        Ok(StreamChunk::Stop { reason: None }),
                        SglangStreamState::Done,
                    ));
                }
                Err(e) => {
                    return Some((
                        Err(RuntimeAdapterError::Network(e.to_string())),
                        SglangStreamState::Done,
                    ));
                }
            }
        },
        SglangStreamState::Done => None,
    }
}

fn extract_openai_delta(value: &Value) -> Option<StreamChunk> {
    let choices = value.get("choices")?.as_array()?;
    let choice = choices.first()?;
    if let Some(reason) = choice.get("finish_reason").and_then(|v| v.as_str()) {
        if !reason.is_empty() && reason != "null" {
            return Some(StreamChunk::Stop {
                reason: Some(reason.to_string()),
            });
        }
    }
    let delta = choice.get("delta")?;
    if let Some(text) = delta.get("content").and_then(|v| v.as_str()) {
        if !text.is_empty() {
            return Some(StreamChunk::TextDelta {
                text: text.to_string(),
            });
        }
    }
    None
}

// ---------------------------------------------------------------------
// Wire types — OpenAI-compatible shape with SGLang's `extra_body`
// passthrough for guided decoding backend selection.
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
    /// SGLang structured-output backend selector. Forwarded as an
    /// extension field on the request body — SGLang's request handler
    /// reads it before dispatching to XGrammar / Outlines / LMFE.
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_body: Option<SglangExtraBody>,
}

#[derive(Debug, Clone, Serialize)]
struct SglangExtraBody {
    backend: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
    #[serde(default)]
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    #[serde(default)]
    data: Vec<ModelEntry>,
}

#[derive(Debug, Deserialize)]
struct ModelEntry {
    #[serde(default)]
    #[allow(dead_code)]
    id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_base_url_is_30000() {
        let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
        // Clear the env so we test the hard-coded default (test runner may
        // inherit ELAI_SGLANG_BASE_URL from outer shell).
        let prev = std::env::var("ELAI_SGLANG_BASE_URL").ok();
        std::env::remove_var("ELAI_SGLANG_BASE_URL");
        let rt = SglangRuntime::new(cfg);
        assert_eq!(rt.base_url(), DEFAULT_BASE_URL);
        if let Some(v) = prev {
            std::env::set_var("ELAI_SGLANG_BASE_URL", v);
        }
    }

    #[test]
    fn env_var_overrides_default_base_url() {
        let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
        std::env::set_var("ELAI_SGLANG_BASE_URL", "http://example.test:31000");
        let rt = SglangRuntime::new(cfg);
        assert_eq!(rt.base_url(), "http://example.test:31000");
        std::env::remove_var("ELAI_SGLANG_BASE_URL");
    }

    #[test]
    fn capabilities_advertise_json_schema_mode() {
        let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
        let rt = SglangRuntime::new(cfg);
        let caps = rt.capabilities();
        assert!(caps.json_schema_mode);
        assert!(caps.supports_streaming);
        assert!(caps.supports_tools);
    }

    #[test]
    fn structured_output_backend_defaults_to_xgrammar() {
        let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
        let rt = SglangRuntime::new(cfg);
        assert_eq!(
            rt.structured_output_backend(),
            StructuredOutputBackend::XGrammar
        );
        assert_eq!(StructuredOutputBackend::XGrammar.as_str(), "xgrammar");
        assert_eq!(StructuredOutputBackend::Outlines.as_str(), "outlines");
        assert_eq!(
            StructuredOutputBackend::Lmfe.as_str(),
            "lm-format-enforcer"
        );
    }

    #[test]
    fn with_runtime_config_overrides_backend_selector() {
        let cfg = SglangRuntimeConfig {
            base_url: "http://sglang.test:30000".to_string(),
            api_key: None,
            request_timeout: Duration::from_secs(45),
            structured_output_backend: StructuredOutputBackend::Outlines,
        };
        let rt = SglangRuntime::with_runtime_config(cfg, Some("test-model".into()));
        assert_eq!(rt.base_url(), "http://sglang.test:30000");
        assert_eq!(rt.model(), "test-model");
        assert_eq!(
            rt.structured_output_backend(),
            StructuredOutputBackend::Outlines
        );
    }

    #[test]
    fn canonical_name_is_sglang() {
        assert_eq!(SglangRuntime::canonical_name(), "sglang");
    }

    #[test]
    fn extract_openai_delta_handles_text() {
        let v: Value = serde_json::from_str(
            r#"{"choices":[{"delta":{"content":"hi"},"finish_reason":null}]}"#,
        )
        .expect("parse");
        match extract_openai_delta(&v) {
            Some(StreamChunk::TextDelta { text }) => assert_eq!(text, "hi"),
            other => panic!("expected TextDelta, got {other:?}"),
        }
    }

    #[test]
    fn registered_event_carries_base_url_and_backend() {
        let event = SglangRuntime::registered_event(
            "http://localhost:30000",
            StructuredOutputBackend::XGrammar,
        );
        match event {
            RuntimeEvent::SglangBackendRegistered {
                base_url,
                structured_output_backend,
            } => {
                assert_eq!(base_url, "http://localhost:30000");
                assert_eq!(structured_output_backend, "xgrammar");
            }
            other => panic!("expected SglangBackendRegistered, got {other:?}"),
        }
    }

    #[test]
    fn observe_cold_start_emits_once_then_never() {
        let seen = AtomicBool::new(false);
        let first = SglangRuntime::observe_cold_start(&seen, 42);
        assert!(first.is_some(), "first call should emit");
        match first {
            Some(RuntimeEvent::RuntimeColdStartObserved { backend, elapsed_ms }) => {
                assert_eq!(backend, RuntimeBackend::Sglang);
                assert_eq!(elapsed_ms, 42);
            }
            other => panic!("wrong event: {other:?}"),
        }
        let second = SglangRuntime::observe_cold_start(&seen, 99);
        assert!(second.is_none(), "second call should not emit");
    }

    #[test]
    fn det_mode_missing_flag_fails() {
        let req = SglangDetModeRequest {
            enable_deterministic_inference: false,
            sampling_seed: Some(42),
            backend: "flashinfer".to_string(),
        };
        assert_eq!(
            validate_sglang_det_mode(&req),
            Err(SglangDetModeError::MissingFlag)
        );
    }

    #[test]
    fn det_mode_missing_sampling_seed_fails() {
        let req = SglangDetModeRequest {
            enable_deterministic_inference: true,
            sampling_seed: None,
            backend: "flashinfer".to_string(),
        };
        assert_eq!(
            validate_sglang_det_mode(&req),
            Err(SglangDetModeError::MissingSamplingSeed)
        );
    }

    #[test]
    fn det_mode_disallowed_backend_fails() {
        let req = SglangDetModeRequest {
            enable_deterministic_inference: true,
            sampling_seed: Some(42),
            backend: "cuda".to_string(),
        };
        assert_eq!(
            validate_sglang_det_mode(&req),
            Err(SglangDetModeError::DisallowedBackend {
                backend: "cuda".to_string()
            })
        );
    }

    #[test]
    fn det_mode_all_three_present_passes() {
        for backend in ["flashinfer", "fa3", "triton"] {
            let req = SglangDetModeRequest {
                enable_deterministic_inference: true,
                sampling_seed: Some(42),
                backend: backend.to_string(),
            };
            assert_eq!(
                validate_sglang_det_mode(&req),
                Ok(()),
                "backend {backend} should pass"
            );
        }
    }

    #[test]
    fn det_mode_backend_as_str_matches_wire_tags() {
        assert_eq!(SglangDetModeBackend::FlashInfer.as_str(), "flashinfer");
        assert_eq!(SglangDetModeBackend::Fa3.as_str(), "fa3");
        assert_eq!(SglangDetModeBackend::Triton.as_str(), "triton");
    }

    #[test]
    fn det_mode_is_allowed_str_rejects_unknown() {
        assert!(SglangDetModeBackend::is_allowed_str("flashinfer"));
        assert!(SglangDetModeBackend::is_allowed_str("fa3"));
        assert!(SglangDetModeBackend::is_allowed_str("triton"));
        assert!(!SglangDetModeBackend::is_allowed_str("cuda"));
        assert!(!SglangDetModeBackend::is_allowed_str("rocm"));
        assert!(!SglangDetModeBackend::is_allowed_str(""));
    }
}
