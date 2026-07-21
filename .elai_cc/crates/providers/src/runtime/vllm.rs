//! `VllmRuntime` — B-37C native HTTP adapter for a local vLLM server.
//!
//! Wire protocol (vLLM ≥0.4 docs, verified 2026-05):
//! - Completion: `POST /v1/chat/completions` — OpenAI-compatible (same
//!   shape as ollama OpenAI-compat path; vLLM is the canonical OpenAI
//!   reference impl for self-hosted serving).
//! - Stream: SSE `data:`-prefixed JSON; final `data: [DONE]`.
//! - Health: `GET /health` (returns 200 when ready) OR
//!   `GET /v1/models` (returns `{"data":[...]}`).

use std::time::Duration;

use futures_util::stream::{self, BoxStream};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::types::{
    unavailable, CompletionConfig, CompletionResponse, Message, MessageRole, RuntimeAdapter,
    RuntimeAdapterError, RuntimeBackend, RuntimeCapabilities, RuntimeFactoryConfig, RuntimeFuture,
    RuntimeHealth, RuntimeHealthFuture, StreamChunk,
};

const BACKEND: RuntimeBackend = RuntimeBackend::Vllm;
const DEFAULT_BASE_URL: &str = "http://localhost:8000";
const DEFAULT_MODEL: &str = "Qwen/Qwen2.5-Coder-7B-Instruct";
const REQUEST_TIMEOUT_SECS: u64 = 120;
const HEALTH_TIMEOUT_SECS: u64 = 5;
const DEFAULT_MAX_TOKENS: u32 = 512;

/// vLLM HTTP runtime adapter.
#[derive(Debug, Clone)]
pub struct VllmRuntime {
    base_url: String,
    model: String,
    http: reqwest::Client,
    health_http: reqwest::Client,
}

impl VllmRuntime {
    #[must_use]
    pub fn new(cfg: RuntimeFactoryConfig) -> Self {
        let base_url = cfg
            .base_url
            .clone()
            .or_else(|| {
                std::env::var("ELAI_VLLM_BASE_URL")
                    .ok()
                    .filter(|s| !s.trim().is_empty())
            })
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

    #[must_use]
    pub fn canonical_name() -> &'static str {
        "vllm"
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

impl RuntimeAdapter for VllmRuntime {
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
            json_schema_mode: true,
            supports_prefill: false,
            supports_caching: false,
            native_thinking_budget: false,
        }
    }

    fn health_check(&self) -> RuntimeHealthFuture<'_> {
        let health_url = self.endpoint("/health");
        let models_url = self.endpoint("/v1/models");
        let client = self.health_http.clone();
        Box::pin(async move {
            // Primary: /health (vLLM's recommended readiness probe).
            match client.get(&health_url).send().await {
                Ok(resp) if resp.status().is_success() => return RuntimeHealth::Ready,
                Ok(_) | Err(_) => {
                    // Fall through to /v1/models.
                }
            }
            match client.get(&models_url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<ModelsResponse>().await {
                        Ok(m) if !m.data.is_empty() => RuntimeHealth::Ready,
                        Ok(_) => unavailable("vllm /v1/models returned empty data array"),
                        Err(e) => unavailable(format!("vllm /v1/models parse failed: {e}")),
                    }
                }
                Ok(resp) => unavailable(format!(
                    "vllm /v1/models returned HTTP {}",
                    resp.status().as_u16()
                )),
                Err(err) => unavailable(format!("vllm unreachable: {err}")),
            }
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
                    "vllm /v1/chat/completions returned HTTP {code}: {body_text}"
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
                .ok_or_else(|| RuntimeAdapterError::Other("vllm returned no choices".into()))?;
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
        };

        Box::pin(stream::unfold(
            VllmStreamState::Connecting {
                client,
                url,
                body: Box::new(body),
            },
            vllm_stream_step,
        ))
    }
}

enum VllmStreamState {
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

async fn vllm_stream_step(
    state: VllmStreamState,
) -> Option<(Result<StreamChunk, RuntimeAdapterError>, VllmStreamState)> {
    match state {
        VllmStreamState::Connecting { client, url, body } => {
            let response = match client.post(&url).json(&*body).send().await {
                Ok(r) => r,
                Err(e) => {
                    return Some((
                        Err(RuntimeAdapterError::Network(e.to_string())),
                        VllmStreamState::Done,
                    ))
                }
            };
            if !response.status().is_success() {
                let code = response.status().as_u16();
                let body_text = response.text().await.unwrap_or_default();
                return Some((
                    Err(RuntimeAdapterError::Other(format!(
                        "vllm stream HTTP {code}: {body_text}"
                    ))),
                    VllmStreamState::Done,
                ));
            }
            Some((
                Ok(StreamChunk::Warning {
                    message: "vllm stream connected".to_string(),
                }),
                VllmStreamState::Streaming {
                    buffer: String::new(),
                    response,
                },
            ))
        }
        VllmStreamState::Streaming {
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
                        VllmStreamState::Done,
                    ));
                }
                match serde_json::from_str::<Value>(&payload) {
                    Ok(value) => {
                        if let Some(chunk) = extract_openai_delta(&value) {
                            return Some((
                                Ok(chunk),
                                VllmStreamState::Streaming { buffer, response },
                            ));
                        }
                        continue;
                    }
                    Err(_) => {
                        return Some((
                            Ok(StreamChunk::Warning {
                                message: format!("non-json stream line: {payload}"),
                            }),
                            VllmStreamState::Streaming { buffer, response },
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
                        VllmStreamState::Done,
                    ));
                }
                Err(e) => {
                    return Some((
                        Err(RuntimeAdapterError::Network(e.to_string())),
                        VllmStreamState::Done,
                    ));
                }
            }
        },
        VllmStreamState::Done => None,
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
// Wire types (subset shared with ollama; duplicated here so the two
// modules stay independent and a future per-backend protocol drift
// doesn't break the other).
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

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn default_base_url_is_8000() {
        let _g = env_lock();
        let prev = std::env::var("ELAI_VLLM_BASE_URL").ok();
        std::env::remove_var("ELAI_VLLM_BASE_URL");
        let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
        let rt = VllmRuntime::new(cfg);
        assert_eq!(rt.base_url(), DEFAULT_BASE_URL);
        match prev {
            Some(v) => std::env::set_var("ELAI_VLLM_BASE_URL", v),
            None => std::env::remove_var("ELAI_VLLM_BASE_URL"),
        }
    }

    #[test]
    fn capabilities_advertise_streaming() {
        let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
        let rt = VllmRuntime::new(cfg);
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
    fn canonical_name_is_vllm() {
        assert_eq!(VllmRuntime::canonical_name(), "vllm");
    }

    #[test]
    fn env_var_overrides_default_base_url() {
        let _g = env_lock();
        let prev = std::env::var("ELAI_VLLM_BASE_URL").ok();
        std::env::set_var("ELAI_VLLM_BASE_URL", "http://vllm-test:8001");
        let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
        let rt = VllmRuntime::new(cfg);
        assert_eq!(rt.base_url(), "http://vllm-test:8001");
        match prev {
            Some(v) => std::env::set_var("ELAI_VLLM_BASE_URL", v),
            None => std::env::remove_var("ELAI_VLLM_BASE_URL"),
        }
    }

    #[test]
    fn cfg_base_url_wins_over_env() {
        let _g = env_lock();
        let prev = std::env::var("ELAI_VLLM_BASE_URL").ok();
        std::env::set_var("ELAI_VLLM_BASE_URL", "http://env-vllm:8001");
        let cfg = RuntimeFactoryConfig {
            base_url: Some("http://cfg-vllm:8002".to_string()),
            ..RuntimeFactoryConfig::for_backend(BACKEND)
        };
        let rt = VllmRuntime::new(cfg);
        assert_eq!(rt.base_url(), "http://cfg-vllm:8002");
        match prev {
            Some(v) => std::env::set_var("ELAI_VLLM_BASE_URL", v),
            None => std::env::remove_var("ELAI_VLLM_BASE_URL"),
        }
    }

    #[test]
    fn empty_env_falls_through_to_default() {
        let _g = env_lock();
        let prev = std::env::var("ELAI_VLLM_BASE_URL").ok();
        std::env::set_var("ELAI_VLLM_BASE_URL", "   ");
        let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
        let rt = VllmRuntime::new(cfg);
        assert_eq!(rt.base_url(), DEFAULT_BASE_URL);
        match prev {
            Some(v) => std::env::set_var("ELAI_VLLM_BASE_URL", v),
            None => std::env::remove_var("ELAI_VLLM_BASE_URL"),
        }
    }
}
