//! E-51: `LlamaCppAdapter` — HTTP adapter for llama-server (GGUF universal).
//!
//! Wraps a llama-server process that speaks /v1/messages (Anthropic-compatible
//! forks) or /v1/chat/completions (OpenAI-compat) with a thin translation shim.
//! The adapter auto-detects the best execution provider (Vulkan / CUDA / Metal /
//! HIP / CPU) via `detect_llamacpp_ep()`.
//!
//! Cross-platform: `new()` always succeeds. llama.cpp ships Win/Mac/Linux/
//! ChromeOS binaries with auto-detected CUDA13.1+Vulkan+HIP+SYCL from
//! ggml-org/llama.cpp b9196+ (Windows prebuilt).
//!
//! Default port: 8181 (distinct from sidecar 8100 and vllm-mlx 8080).

use std::time::Duration;

use futures_util::stream::{self, BoxStream};
use serde_json::Value;

use super::event_sink::{runtime_event_sink, RuntimeEvent};
use super::platform_detect::{detect_llamacpp_ep, LlamaCppEp};
use super::types::{
    unavailable, CompletionConfig, CompletionResponse, Message, MessageRole, RuntimeAdapter,
    RuntimeAdapterError, RuntimeBackend, RuntimeCapabilities, RuntimeFactoryConfig, RuntimeFuture,
    RuntimeHealth, RuntimeHealthFuture, StreamChunk,
};
use crate::types::{InputContentBlock, InputMessage, MessageRequest, MessageResponse, SystemBlock};

const DEFAULT_BASE_URL: &str = "http://localhost:8181";
const DEFAULT_MODEL: &str = "llama-cpp-default";
const REQUEST_TIMEOUT_SECS: u64 = 120;
const HEALTH_TIMEOUT_SECS: u64 = 5;
const DEFAULT_MAX_TOKENS: u32 = 512;

/// HTTP runtime adapter for a llama-server subprocess.
/// Cross-platform; `new()` always succeeds.
#[derive(Debug, Clone)]
pub struct LlamaCppAdapter {
    base_url: String,
    model: String,
    http: reqwest::Client,
    health_http: reqwest::Client,
    max_tokens: u32,
    ep: LlamaCppEp,
}

impl LlamaCppAdapter {
    /// Build the adapter from a `RuntimeFactoryConfig`.
    /// Always succeeds — llama.cpp runs on all platforms.
    #[must_use]
    pub fn new(cfg: RuntimeFactoryConfig) -> Self {
        let ep = detect_llamacpp_ep();
        tracing::info!(ep = ?ep, "LlamaCppAdapter: EP detected");

        runtime_event_sink().record(RuntimeEvent::CliSubprocessSpawned {
            backend: RuntimeBackend::LlamaCpp,
            pid: 0,
            argv_redacted: vec!["llama-server".into(), "(not spawned yet)".into()],
        });

        let base_url = cfg
            .base_url
            .clone()
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let model = cfg
            .model
            .clone()
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());
        let max_tokens = cfg
            .sidecar
            .as_ref()
            .and_then(|s| s.max_tokens)
            .unwrap_or(DEFAULT_MAX_TOKENS);

        // Build HTTP clients with graceful fallback on builder failure.
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "LlamaCppAdapter: reqwest builder failed, using default");
                reqwest::Client::new()
            });
        let health_http = reqwest::Client::builder()
            .timeout(Duration::from_secs(HEALTH_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            base_url,
            model,
            http,
            health_http,
            max_tokens,
            ep,
        }
    }

    /// The detected execution provider.
    #[must_use]
    pub fn ep(&self) -> &LlamaCppEp {
        &self.ep
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
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
            // I-26: effort is a cloud-only field; llama.cpp backend never uses it.
            effort: None,
            chat_template_kwargs: config.chat_template_kwargs.clone(),
        }
    }
}

impl RuntimeAdapter for LlamaCppAdapter {
    fn backend(&self) -> RuntimeBackend {
        RuntimeBackend::LlamaCpp
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        // Conservative defaults — llama.cpp doesn't support tools/vision natively.
        RuntimeCapabilities {
            supports_tools: false,
            supports_streaming: false,
            supports_vision: false,
            max_context: 32_768,
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
                    "llama-server /health returned HTTP {}",
                    resp.status().as_u16()
                )),
                Err(err) => unavailable(format!("llama-server /health unreachable: {err}")),
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
                    "llama-server /v1/messages returned HTTP {status}: {body}"
                )));
            }

            let raw = response
                .bytes()
                .await
                .map_err(|e| RuntimeAdapterError::Network(e.to_string()))?;

            // Try strict Anthropic MessageResponse; fall back to best-effort extraction.
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
                        cache_creation_input_tokens: parsed.usage.cache_creation_input_tokens,
                        cache_read_input_tokens: parsed.usage.cache_read_input_tokens,
                        model: parsed.model,
                    })
                }
                Err(_) => {
                    let value: Value = serde_json::from_slice(&raw).map_err(|e| {
                        RuntimeAdapterError::Other(format!(
                            "llama-server returned unparseable response: {e}"
                        ))
                    })?;
                    let text = extract_text_from_value(&value).unwrap_or_default();
                    Ok(CompletionResponse {
                        text,
                        stop_reason: None,
                        usage: (0, 0),
                        cache_creation_input_tokens: 0,
                        cache_read_input_tokens: 0,
                        model: request.model.clone(),
                    })
                }
            }
        })
    }

    fn stream(
        &self,
        _messages: Vec<Message>,
        _config: CompletionConfig,
    ) -> BoxStream<'static, Result<StreamChunk, RuntimeAdapterError>> {
        // Streaming deferred to E-51.2.
        Box::pin(stream::once(async {
            Err(RuntimeAdapterError::NotImplemented {
                backend: RuntimeBackend::LlamaCpp,
                hint: "streaming deferred to E-51.2",
            })
        }))
    }
}

/// Best-effort text extraction from a non-standard JSON response shape.
fn extract_text_from_value(value: &Value) -> Option<String> {
    if let Some(content) = value.get("content").and_then(|c| c.as_array()) {
        for block in content {
            if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                return Some(text.to_string());
            }
        }
    }
    if let Some(text) = value
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|t| t.as_str())
    {
        return Some(text.to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn llamacpp_adapter_new_always_succeeds() {
        let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::LlamaCpp);
        let adapter = LlamaCppAdapter::new(cfg);
        assert_eq!(adapter.backend(), RuntimeBackend::LlamaCpp);
    }

    #[test]
    fn llamacpp_capabilities_non_zero() {
        let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::LlamaCpp);
        let adapter = LlamaCppAdapter::new(cfg);
        let caps = adapter.capabilities();
        assert!(caps.max_context > 0);
        assert!(caps.max_output > 0);
    }

    #[test]
    fn extract_text_anthropic_shape() {
        let val = serde_json::json!({
            "content": [{"type": "text", "text": "hello"}]
        });
        assert_eq!(extract_text_from_value(&val), Some("hello".to_string()));
    }
}
