//! E-51: `MlxAdapter` — HTTP adapter for vllm-mlx on Darwin+arm64.
//!
//! Wraps a vllm-mlx process that already speaks Anthropic /v1/messages
//! (github.com/waybarrios/vllm-mlx; verified via README claims 2026-05).
//! The lifecycle is identical to `SidecarRuntime` — we do NOT spawn the
//! subprocess; the operator starts vllm-mlx separately (or via the future
//! E-51.2 auto-spawn helper). This adapter is a HTTP client only.
//!
//! Platform gate: `detect_mlx_capable()` is called in `new()`. If the
//! host is `NotApple` or `AppleIntel`, the constructor returns
//! `RuntimeAdapterError::Other` so `RuntimeOrchestrator::ensure_adapter`
//! logs a health failure and falls back to the next chain entry via
//! `is_recoverable`.
//!
//! Default port: 8080 (vllm-mlx default differs from sidecar's 8100).

use std::time::Duration;

use futures_util::stream::{self, BoxStream};
use serde_json::Value;

use super::event_sink::{runtime_event_sink, RuntimeEvent};
use super::platform_detect::{detect_mlx_capable, MlxCapability};
use super::types::{
    unavailable, CompletionConfig, CompletionResponse, Message, MessageRole, RuntimeAdapter,
    RuntimeAdapterError, RuntimeBackend, RuntimeCapabilities, RuntimeFactoryConfig, RuntimeFuture,
    RuntimeHealth, RuntimeHealthFuture, StreamChunk,
};
use crate::types::{InputContentBlock, InputMessage, MessageRequest, MessageResponse, SystemBlock};

const DEFAULT_BASE_URL: &str = "http://localhost:8080";
const DEFAULT_MODEL: &str = "mlx-default";
const REQUEST_TIMEOUT_SECS: u64 = 120;
const HEALTH_TIMEOUT_SECS: u64 = 5;
const DEFAULT_MAX_TOKENS: u32 = 512;
/// vllm-mlx supports longer context than the HF NF4 sidecar.
const DEFAULT_MAX_CONTEXT: u32 = 32_000;

/// HTTP runtime adapter for a vllm-mlx subprocess on Darwin+arm64.
#[derive(Debug, Clone)]
pub struct MlxAdapter {
    base_url: String,
    model: String,
    http: reqwest::Client,
    health_http: reqwest::Client,
    max_tokens: u32,
}

impl MlxAdapter {
    /// Build the adapter from a `RuntimeFactoryConfig`.
    ///
    /// Returns `Err(RuntimeAdapterError::Other)` on non-Apple-Silicon
    /// platforms so the `RuntimeOrchestrator` can fall back gracefully.
    pub fn new(cfg: RuntimeFactoryConfig) -> Result<Self, RuntimeAdapterError> {
        let cap = detect_mlx_capable();
        match &cap {
            MlxCapability::NotApple => {
                return Err(RuntimeAdapterError::Other(
                    "MLX requires Darwin+arm64 (Apple Silicon). \
                     Current platform: NotApple (non-Darwin)"
                        .to_string(),
                ));
            }
            MlxCapability::AppleIntel => {
                return Err(RuntimeAdapterError::Other(
                    "MLX requires Darwin+arm64 (Apple Silicon M-series). \
                     Current platform: AppleIntel (Intel Mac, arm64 required)"
                        .to_string(),
                ));
            }
            MlxCapability::AppleSilicon {
                chip_family,
                gpu_cores,
            } => {
                tracing::info!(
                    chip_family = ?chip_family,
                    gpu_cores,
                    "MlxAdapter: Apple Silicon detected"
                );
                runtime_event_sink().record(RuntimeEvent::CliSubprocessSpawned {
                    backend: RuntimeBackend::Mlx,
                    pid: 0,
                    argv_redacted: vec!["vllm-mlx".into(), "(not spawned yet)".into()],
                });
            }
        }

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

        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .map_err(|e| RuntimeAdapterError::Network(e.to_string()))?;
        let health_http = reqwest::Client::builder()
            .timeout(Duration::from_secs(HEALTH_TIMEOUT_SECS))
            .build()
            .map_err(|e| RuntimeAdapterError::Network(e.to_string()))?;

        Ok(Self {
            base_url,
            model,
            http,
            health_http,
            max_tokens,
        })
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
            // I-26: effort is a cloud-only field; MLX backend never uses it.
            effort: None,
            chat_template_kwargs: config.chat_template_kwargs.clone(),
        }
    }
}

impl RuntimeAdapter for MlxAdapter {
    fn backend(&self) -> RuntimeBackend {
        RuntimeBackend::Mlx
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        // vllm-mlx targets continuous batching + Apple Silicon Metal.
        // Tool use / vision deferred; streaming deferred to E-51.2.
        RuntimeCapabilities {
            supports_tools: false,
            supports_streaming: false,
            supports_vision: false,
            max_context: DEFAULT_MAX_CONTEXT,
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
                    "vllm-mlx /health returned HTTP {}",
                    resp.status().as_u16()
                )),
                Err(err) => unavailable(format!("vllm-mlx /health unreachable: {err}")),
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
                    "vllm-mlx /v1/messages returned HTTP {status}: {body}"
                )));
            }

            let raw = response
                .bytes()
                .await
                .map_err(|e| RuntimeAdapterError::Network(e.to_string()))?;

            // Try strict Anthropic MessageResponse first; fall back to
            // best-effort JSON extraction for minor protocol drift.
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
                            "vllm-mlx returned unparseable response: {e}"
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
        // Streaming deferred to E-51.2 (requires vllm-mlx SSE consumer).
        Box::pin(stream::once(async {
            Err(RuntimeAdapterError::NotImplemented {
                backend: RuntimeBackend::Mlx,
                hint: "streaming deferred to E-51.2 (requires vllm-mlx SSE consumer)",
            })
        }))
    }
}

/// Best-effort text extraction from a non-standard JSON response shape.
fn extract_text_from_value(value: &Value) -> Option<String> {
    // Try Anthropic content[0].text
    if let Some(content) = value.get("content").and_then(|c| c.as_array()) {
        for block in content {
            if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                return Some(text.to_string());
            }
        }
    }
    // Try OpenAI choices[0].message.content
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

    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    #[test]
    fn mlx_adapter_new_fails_on_non_apple_silicon() {
        let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::Mlx);
        let result = MlxAdapter::new(cfg);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("MLX") || msg.contains("Apple"),
            "error must mention MLX/Apple, got: {msg}"
        );
    }

    #[test]
    fn extract_text_anthropic_shape() {
        let val = serde_json::json!({
            "content": [{"type": "text", "text": "hello"}]
        });
        assert_eq!(extract_text_from_value(&val), Some("hello".to_string()));
    }

    #[test]
    fn extract_text_openai_shape() {
        let val = serde_json::json!({
            "choices": [{"message": {"content": "world"}}]
        });
        assert_eq!(extract_text_from_value(&val), Some("world".to_string()));
    }
}
