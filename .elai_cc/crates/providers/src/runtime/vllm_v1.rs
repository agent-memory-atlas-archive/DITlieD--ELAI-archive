//! J-19: `VllmV1Runtime` — vLLM V1 native `/v1/messages` adapter.
//!
//! Speaks the Anthropic Messages wire shape (same as `LlamaCppAdapter`)
//! against a vLLM V1 server. Opt-in only via `resolve_backend("vllm-v1")`
//! or `resolve_backend("vllm_v1")`; never added to `PerRoleRuntime` defaults.
//!
//! Distinguishing flags (read from env at `new` time):
//! - `ELAI_VLLM_V1_BATCH_INVARIANT` — batch-invariant serving mode.
//! - `ELAI_VLLM_V1_PREFIX_CACHING`  — APC (automatic prefix caching) claim.
//!
//! Health probe mirrors `vllm.rs`: GET /health, fallback GET /v1/models.
//! Live CUDA/RunPod measure leg is W-10; the adapter itself is
//! offline-buildable (no GPU code paths in Rust).

use std::time::Duration;

use futures_util::stream::{self, BoxStream};
use serde_json::Value;

use super::types::{
    unavailable, CompletionConfig, CompletionResponse, Message, MessageRole, RuntimeAdapter,
    RuntimeAdapterError, RuntimeBackend, RuntimeCapabilities, RuntimeFactoryConfig, RuntimeFuture,
    RuntimeHealth, RuntimeHealthFuture, StreamChunk,
};
use crate::types::{InputContentBlock, InputMessage, MessageRequest, MessageResponse, SystemBlock};

const BACKEND: RuntimeBackend = RuntimeBackend::VllmV1;
const DEFAULT_BASE_URL: &str = "http://localhost:8000";
const DEFAULT_MODEL: &str = "Qwen/Qwen2.5-Coder-7B-Instruct";
const REQUEST_TIMEOUT_SECS: u64 = 120;
const HEALTH_TIMEOUT_SECS: u64 = 5;
const DEFAULT_MAX_TOKENS: u32 = 512;

/// vLLM V1 HTTP runtime adapter (Anthropic `/v1/messages` wire shape).
#[derive(Debug, Clone)]
pub struct VllmV1Runtime {
    base_url: String,
    model: String,
    http: reqwest::Client,
    health_http: reqwest::Client,
    batch_invariant: bool,
    prefix_caching: bool,
}

fn env_truthy(name: &str) -> bool {
    matches!(
        std::env::var(name).ok().as_deref().map(str::to_ascii_lowercase).as_deref(),
        Some("1") | Some("true") | Some("yes") | Some("on")
    )
}

impl VllmV1Runtime {
    /// Build the adapter from a `RuntimeFactoryConfig`. Always succeeds
    /// (constructs HTTP clients with graceful fallback on builder failure,
    /// mirroring `LlamaCppAdapter::new`).
    #[must_use]
    pub fn new(cfg: RuntimeFactoryConfig) -> Self {
        let base_url = cfg
            .base_url
            .clone()
            .or_else(|| {
                std::env::var("ELAI_VLLM_V1_BASE_URL")
                    .ok()
                    .filter(|s| !s.trim().is_empty())
            })
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
        let batch_invariant = env_truthy("ELAI_VLLM_V1_BATCH_INVARIANT");
        let prefix_caching = env_truthy("ELAI_VLLM_V1_PREFIX_CACHING");
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "VllmV1Runtime: reqwest builder failed, using default");
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
            batch_invariant,
            prefix_caching,
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
    pub fn batch_invariant(&self) -> bool {
        self.batch_invariant
    }

    #[must_use]
    pub fn prefix_caching(&self) -> bool {
        self.prefix_caching
    }

    #[must_use]
    pub fn canonical_name() -> &'static str {
        "vllm_v1"
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
            DEFAULT_MAX_TOKENS
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
            effort: None,
            chat_template_kwargs: config.chat_template_kwargs.clone(),
        }
    }
}

impl RuntimeAdapter for VllmV1Runtime {
    fn backend(&self) -> RuntimeBackend {
        BACKEND
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        // Conservative defaults mirroring VllmRuntime; APC claim surfaces
        // via supports_caching = true (J-19 batch-invariant + prefix-cache).
        RuntimeCapabilities {
            supports_tools: true,
            supports_streaming: true,
            supports_vision: false,
            max_context: 32_768,
            max_output: DEFAULT_MAX_TOKENS,
            json_schema_mode: true,
            supports_prefill: false,
            supports_caching: self.prefix_caching,
            native_thinking_budget: false,
        }
    }

    fn health_check(&self) -> RuntimeHealthFuture<'_> {
        let health_url = self.endpoint("/health");
        let models_url = self.endpoint("/v1/models");
        let client = self.health_http.clone();
        Box::pin(async move {
            match client.get(&health_url).send().await {
                Ok(resp) if resp.status().is_success() => RuntimeHealth::Ready,
                Ok(resp) => {
                    let code = resp.status().as_u16();
                    match client.get(&models_url).send().await {
                        Ok(r) if r.status().is_success() => RuntimeHealth::Ready,
                        Ok(_) | Err(_) => unavailable(format!(
                            "vllm-v1 /health returned HTTP {code}; /v1/models fallback failed (j19-vllm-v1)"
                        )),
                    }
                }
                Err(err) => unavailable(format!(
                    "vllm-v1 /health unreachable: {err} (j19-vllm-v1)"
                )),
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
                    "vllm-v1 /v1/messages returned HTTP {status}: {body}"
                )));
            }

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
                            "vllm-v1 returned unparseable response: {e}"
                        ))
                    })?;
                    let text = extract_text_from_value(&value).unwrap_or_default();
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

    fn stream(
        &self,
        _messages: Vec<Message>,
        _config: CompletionConfig,
    ) -> BoxStream<'static, Result<StreamChunk, RuntimeAdapterError>> {
        // Streaming deferred to a later J-19 sub-slice; the V1 server's
        // SSE /v1/messages stream shape needs a live measure leg (W-10).
        Box::pin(stream::once(async {
            Err(RuntimeAdapterError::NotImplemented {
                backend: BACKEND,
                hint: "vllm-v1 streaming deferred to J-19 sub-slice (W-10 measure leg)",
            })
        }))
    }
}

/// Best-effort text extraction from a non-standard JSON response shape
/// (mirrors `llama_cpp::extract_text_from_value`).
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

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn new_always_succeeds() {
        let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
        let rt = VllmV1Runtime::new(cfg);
        assert_eq!(rt.backend(), BACKEND);
        assert_eq!(rt.model(), DEFAULT_MODEL);
    }

    #[test]
    fn default_base_url_is_8000() {
        let _g = env_lock();
        let prev_v1 = std::env::var("ELAI_VLLM_V1_BASE_URL").ok();
        let prev = std::env::var("ELAI_VLLM_BASE_URL").ok();
        std::env::remove_var("ELAI_VLLM_V1_BASE_URL");
        std::env::remove_var("ELAI_VLLM_BASE_URL");
        let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
        let rt = VllmV1Runtime::new(cfg);
        assert_eq!(rt.base_url(), DEFAULT_BASE_URL);
        for (k, v) in [
            ("ELAI_VLLM_V1_BASE_URL", prev_v1),
            ("ELAI_VLLM_BASE_URL", prev),
        ] {
            match v {
                Some(val) => std::env::set_var(k, val),
                None => std::env::remove_var(k),
            }
        }
    }

    #[test]
    fn v1_base_url_wins_over_generic() {
        let _g = env_lock();
        let prev_v1 = std::env::var("ELAI_VLLM_V1_BASE_URL").ok();
        let prev = std::env::var("ELAI_VLLM_BASE_URL").ok();
        std::env::set_var("ELAI_VLLM_V1_BASE_URL", "http://v1:8001");
        std::env::set_var("ELAI_VLLM_BASE_URL", "http://generic:8002");
        let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
        let rt = VllmV1Runtime::new(cfg);
        assert_eq!(rt.base_url(), "http://v1:8001");
        for (k, v) in [
            ("ELAI_VLLM_V1_BASE_URL", prev_v1),
            ("ELAI_VLLM_BASE_URL", prev),
        ] {
            match v {
                Some(val) => std::env::set_var(k, val),
                None => std::env::remove_var(k),
            }
        }
    }

    #[test]
    fn cfg_base_url_wins_over_env() {
        let _g = env_lock();
        let prev_v1 = std::env::var("ELAI_VLLM_V1_BASE_URL").ok();
        std::env::set_var("ELAI_VLLM_V1_BASE_URL", "http://env-vllm-v1:8001");
        let cfg = RuntimeFactoryConfig {
            base_url: Some("http://cfg-vllm-v1:8002".to_string()),
            ..RuntimeFactoryConfig::for_backend(BACKEND)
        };
        let rt = VllmV1Runtime::new(cfg);
        assert_eq!(rt.base_url(), "http://cfg-vllm-v1:8002");
        match prev_v1 {
            Some(v) => std::env::set_var("ELAI_VLLM_V1_BASE_URL", v),
            None => std::env::remove_var("ELAI_VLLM_V1_BASE_URL"),
        }
    }

    #[test]
    fn batch_invariant_env_truthy() {
        let _g = env_lock();
        let prev = std::env::var("ELAI_VLLM_V1_BATCH_INVARIANT").ok();
        std::env::set_var("ELAI_VLLM_V1_BATCH_INVARIANT", "true");
        let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
        let rt = VllmV1Runtime::new(cfg);
        assert!(rt.batch_invariant());
        match prev {
            Some(v) => std::env::set_var("ELAI_VLLM_V1_BATCH_INVARIANT", v),
            None => std::env::remove_var("ELAI_VLLM_V1_BATCH_INVARIANT"),
        }
    }

    #[test]
    fn prefix_caching_drives_capabilities() {
        let _g = env_lock();
        let prev = std::env::var("ELAI_VLLM_V1_PREFIX_CACHING").ok();
        std::env::set_var("ELAI_VLLM_V1_PREFIX_CACHING", "1");
        let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
        let rt = VllmV1Runtime::new(cfg);
        let caps = rt.capabilities();
        assert!(caps.supports_caching);
        assert!(caps.supports_tools);
        assert!(caps.supports_streaming);
        match prev {
            Some(v) => std::env::set_var("ELAI_VLLM_V1_PREFIX_CACHING", v),
            None => std::env::remove_var("ELAI_VLLM_V1_PREFIX_CACHING"),
        }
    }

    #[test]
    fn canonical_name_is_vllm_v1() {
        assert_eq!(VllmV1Runtime::canonical_name(), "vllm_v1");
    }

    #[test]
    fn extract_text_anthropic_shape() {
        let val = serde_json::json!({
            "content": [{"type": "text", "text": "hello"}]
        });
        assert_eq!(extract_text_from_value(&val), Some("hello".to_string()));
    }
}
