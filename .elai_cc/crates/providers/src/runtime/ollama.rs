//! `OllamaRuntime` — B-37C native HTTP adapter for the local Ollama
//! daemon.
//!
//! Wire protocol (verified at runtime, ollama.com docs as of 2026-05):
//! - Primary: `POST /v1/chat/completions` — OpenAI-compatible shape.
//!   Stream chunks are SSE lines prefixed with `data: ` carrying an
//!   OpenAI delta JSON. A final `data: [DONE]` line terminates.
//! - Fallback: `POST /api/chat` — native Ollama shape. Stream is bare
//!   NDJSON (one JSON object per line) with `"done": true` at the end.
//!   Used only if the OpenAI-compat path 404s.
//! - Health: `GET /api/tags` — returns `{ "models": [...] }`. We
//!   require HTTP 200 AND a non-empty `models` array.
//!
//! Defaults:
//! - Base URL: `RuntimeFactoryConfig.base_url`, else env `OLLAMA_HOST`,
//!   else `http://localhost:11434`.
//! - Request timeout: 120s. Health timeout: 5s.

use std::time::Duration;

use futures_util::stream::{self, BoxStream};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::types::{
    unavailable, CompletionConfig, CompletionResponse, Message, MessageRole, RuntimeAdapter,
    RuntimeAdapterError, RuntimeBackend, RuntimeCapabilities, RuntimeFactoryConfig, RuntimeFuture,
    RuntimeHealth, RuntimeHealthFuture, StreamChunk,
};

const BACKEND: RuntimeBackend = RuntimeBackend::Ollama;
const DEFAULT_BASE_URL: &str = "http://localhost:11434";
const DEFAULT_MODEL: &str = "llama3.1";
const REQUEST_TIMEOUT_SECS: u64 = 120;
const HEALTH_TIMEOUT_SECS: u64 = 5;
const DEFAULT_MAX_TOKENS: u32 = 512;

/// Ollama HTTP runtime adapter.
#[derive(Debug, Clone)]
pub struct OllamaRuntime {
    base_url: String,
    model: String,
    http: reqwest::Client,
    health_http: reqwest::Client,
}

impl OllamaRuntime {
    /// Construct from `RuntimeFactoryConfig`.
    #[must_use]
    pub fn new(cfg: RuntimeFactoryConfig) -> Self {
        let base_url = cfg
            .base_url
            .clone()
            .or_else(|| std::env::var("OLLAMA_HOST").ok())
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
        Self {
            base_url,
            model,
            http,
            health_http,
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

    /// Canonical backend identifier (lowercase snake_case). Mirrors the
    /// canonical name in `RuntimeBackend::canonical_name`.
    #[must_use]
    pub fn canonical_name() -> &'static str {
        "ollama"
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }

    fn build_openai_messages(messages: &[Message]) -> Vec<OpenAiMessage> {
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

impl RuntimeAdapter for OllamaRuntime {
    fn backend(&self) -> RuntimeBackend {
        BACKEND
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities {
            supports_tools: true,
            supports_streaming: true,
            supports_vision: false,
            max_context: 32_768,
            max_output: DEFAULT_MAX_TOKENS,
            json_schema_mode: false,
            supports_prefill: false,
            supports_caching: false,
            native_thinking_budget: false,
        }
    }

    fn health_check(&self) -> RuntimeHealthFuture<'_> {
        let url = self.endpoint("/api/tags");
        let client = self.health_http.clone();
        Box::pin(async move {
            match client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<TagsResponse>().await {
                        Ok(tags) if !tags.models.is_empty() => RuntimeHealth::Ready,
                        Ok(_) => unavailable("ollama /api/tags returned empty models list"),
                        Err(e) => unavailable(format!(
                            "ollama /api/tags response parse failed: {e}"
                        )),
                    }
                }
                Ok(resp) => unavailable(format!(
                    "ollama /api/tags returned HTTP {}",
                    resp.status().as_u16()
                )),
                Err(err) => unavailable(format!("ollama /api/tags unreachable: {err}")),
            }
        })
    }

    fn complete(
        &self,
        messages: Vec<Message>,
        config: CompletionConfig,
    ) -> RuntimeFuture<'_, CompletionResponse> {
        let primary_url = self.endpoint("/v1/chat/completions");
        let fallback_url = self.endpoint("/api/chat");
        let client = self.http.clone();
        let default_model = self.model.clone();
        Box::pin(async move {
            let model = config.model.clone().unwrap_or(default_model);
            let max_tokens = if config.max_tokens == 0 {
                DEFAULT_MAX_TOKENS
            } else {
                config.max_tokens
            };
            let openai_messages = Self::build_openai_messages(&messages);

            let body = OpenAiChatRequest {
                model: model.clone(),
                messages: openai_messages.clone(),
                max_tokens: Some(max_tokens),
                temperature: Some(config.temperature),
                stream: false,
                stop: if config.stop_sequences.is_empty() {
                    None
                } else {
                    Some(config.stop_sequences.clone())
                },
            };

            let response = client
                .post(&primary_url)
                .json(&body)
                .send()
                .await
                .map_err(|e| RuntimeAdapterError::Network(e.to_string()))?;

            if response.status() == reqwest::StatusCode::NOT_FOUND {
                tracing::debug!(
                    target: "elai_providers::runtime::ollama",
                    "/v1/chat/completions 404; falling back to /api/chat"
                );
                let native_body = NativeChatRequest {
                    model: model.clone(),
                    messages: openai_messages,
                    stream: false,
                };
                let native_resp = client
                    .post(&fallback_url)
                    .json(&native_body)
                    .send()
                    .await
                    .map_err(|e| RuntimeAdapterError::Network(e.to_string()))?;
                if !native_resp.status().is_success() {
                    let code = native_resp.status().as_u16();
                    let body = native_resp.text().await.unwrap_or_default();
                    return Err(RuntimeAdapterError::Other(format!(
                        "ollama /api/chat returned HTTP {code}: {body}"
                    )));
                }
                let parsed: NativeChatResponse = native_resp
                    .json()
                    .await
                    .map_err(|e| RuntimeAdapterError::Network(e.to_string()))?;
                return Ok(CompletionResponse {
                    text: parsed.message.content,
                    stop_reason: parsed.done_reason,
                    usage: (
                        parsed.prompt_eval_count.unwrap_or(0),
                        parsed.eval_count.unwrap_or(0),
                    ),
                    model,
                });
            }

            if !response.status().is_success() {
                let code = response.status().as_u16();
                let body = response.text().await.unwrap_or_default();
                return Err(RuntimeAdapterError::Other(format!(
                    "ollama /v1/chat/completions returned HTTP {code}: {body}"
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
                .ok_or_else(|| RuntimeAdapterError::Other("ollama returned no choices".into()))?;
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
        let primary_url = self.endpoint("/v1/chat/completions");
        let client = self.http.clone();
        let default_model = self.model.clone();
        let model = config.model.clone().unwrap_or(default_model);
        let max_tokens = if config.max_tokens == 0 {
            DEFAULT_MAX_TOKENS
        } else {
            config.max_tokens
        };
        let openai_messages = Self::build_openai_messages(&messages);
        let body = OpenAiChatRequest {
            model,
            messages: openai_messages,
            max_tokens: Some(max_tokens),
            temperature: Some(config.temperature),
            stream: true,
            stop: if config.stop_sequences.is_empty() {
                None
            } else {
                Some(config.stop_sequences.clone())
            },
        };

        Box::pin(stream::unfold(
            OllamaStreamState::Connecting {
                client,
                url: primary_url,
                body: Box::new(body),
            },
            ollama_stream_step,
        ))
    }
}

/// SSE / NDJSON streaming state machine. Stored across `unfold` ticks.
/// We hold the live `reqwest::Response` (after a successful POST) and
/// pull byte chunks via `Response::chunk()`. No external `bytes::Bytes`
/// import needed.
enum OllamaStreamState {
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

async fn ollama_stream_step(
    state: OllamaStreamState,
) -> Option<(
    Result<StreamChunk, RuntimeAdapterError>,
    OllamaStreamState,
)> {
    match state {
        OllamaStreamState::Connecting { client, url, body } => {
            let response = match client.post(&url).json(&*body).send().await {
                Ok(r) => r,
                Err(e) => {
                    return Some((
                        Err(RuntimeAdapterError::Network(e.to_string())),
                        OllamaStreamState::Done,
                    ))
                }
            };
            if !response.status().is_success() {
                let code = response.status().as_u16();
                let body_text = response.text().await.unwrap_or_default();
                return Some((
                    Err(RuntimeAdapterError::Other(format!(
                        "ollama stream HTTP {code}: {body_text}"
                    ))),
                    OllamaStreamState::Done,
                ));
            }
            Some((
                Ok(StreamChunk::Warning {
                    message: "ollama stream connected".to_string(),
                }),
                OllamaStreamState::Streaming {
                    buffer: String::new(),
                    response,
                },
            ))
        }
        OllamaStreamState::Streaming {
            mut buffer,
            mut response,
        } => {
            loop {
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
                            OllamaStreamState::Done,
                        ));
                    }
                    match serde_json::from_str::<Value>(&payload) {
                        Ok(value) => {
                            if let Some(chunk) = extract_openai_delta(&value) {
                                return Some((
                                    Ok(chunk),
                                    OllamaStreamState::Streaming { buffer, response },
                                ));
                            }
                            continue;
                        }
                        Err(_) => {
                            return Some((
                                Ok(StreamChunk::Warning {
                                    message: format!("non-json stream line: {payload}"),
                                }),
                                OllamaStreamState::Streaming { buffer, response },
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
                        let tail = buffer.trim().to_string();
                        if tail.is_empty() {
                            return Some((
                                Ok(StreamChunk::Stop { reason: None }),
                                OllamaStreamState::Done,
                            ));
                        }
                        let payload = tail.strip_prefix("data: ").unwrap_or(&tail).to_string();
                        if payload == "[DONE]" {
                            return Some((
                                Ok(StreamChunk::Stop { reason: None }),
                                OllamaStreamState::Done,
                            ));
                        }
                        if let Ok(value) = serde_json::from_str::<Value>(&payload) {
                            if let Some(chunk) = extract_openai_delta(&value) {
                                return Some((Ok(chunk), OllamaStreamState::Done));
                            }
                        }
                        return Some((
                            Ok(StreamChunk::Stop { reason: None }),
                            OllamaStreamState::Done,
                        ));
                    }
                    Err(e) => {
                        return Some((
                            Err(RuntimeAdapterError::Network(e.to_string())),
                            OllamaStreamState::Done,
                        ));
                    }
                }
            }
        }
        OllamaStreamState::Done => None,
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
// Wire types
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

#[derive(Debug, Serialize)]
struct NativeChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct NativeChatResponse {
    message: OpenAiMessage,
    #[serde(default)]
    done_reason: Option<String>,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<TagModel>,
}

#[derive(Debug, Deserialize)]
struct TagModel {
    #[serde(default)]
    #[allow(dead_code)]
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_base_url_is_local_11434() {
        let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
        let rt = OllamaRuntime::new(cfg);
        // OLLAMA_HOST may be set in some test envs; accept either the
        // explicit default or any value derived from it.
        assert!(
            rt.base_url() == DEFAULT_BASE_URL || std::env::var("OLLAMA_HOST").is_ok(),
            "unexpected base_url {}",
            rt.base_url()
        );
    }

    #[test]
    fn capabilities_advertise_streaming() {
        let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
        let rt = OllamaRuntime::new(cfg);
        let caps = rt.capabilities();
        assert!(caps.supports_streaming);
        assert!(caps.supports_tools);
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
    fn extract_openai_delta_handles_stop() {
        let v: Value = serde_json::from_str(
            r#"{"choices":[{"delta":{},"finish_reason":"stop"}]}"#,
        )
        .expect("parse");
        match extract_openai_delta(&v) {
            Some(StreamChunk::Stop { reason }) => assert_eq!(reason, Some("stop".into())),
            other => panic!("expected Stop, got {other:?}"),
        }
    }

    #[test]
    fn canonical_name_is_ollama() {
        assert_eq!(OllamaRuntime::canonical_name(), "ollama");
    }
}
