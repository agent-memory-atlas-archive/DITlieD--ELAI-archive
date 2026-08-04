//! J-54 FreePool RuntimeAdapter — multi-provider free-tier pool with
//! fallback chain, quota tracking, and Pollinations keyless overflow.
//!
//! The adapter implements `RuntimeAdapter::complete` by:
//! 1. Selecting a deployment via `select_with_heal` (fallback chain
//!    + quota filtering + bounded retry on PoolExhausted).
//! 2. POSTing an OpenAI-compat chat/completions request via reqwest.
//! 3. On 429: calling `QuotaTracker::note_429` and retrying with the next
//!    deployment.
//! 4. On success: calling `QuotaTracker::record_success` and returning the
//!    completion response.
//!
//! Streaming is not supported (returns `NotImplemented`).

pub mod catalog;
pub mod dispatch;
pub mod heal;
pub mod quota;

pub use catalog::{load_free_pool_catalog, FreePoolCatalogEntry};
pub use dispatch::{Deployment, FreePoolDispatcher, PoolExhausted, ProviderGroup};
pub use heal::{heal_or_spill, select_with_heal, HealConfig, HealOutcome, SelectError};
pub use quota::{KeyId, QuotaLimits, QuotaTracker};

use std::path::PathBuf;
use std::sync::Arc;

use futures_util::stream::{self, BoxStream};
use serde_json::json;

use super::types::{
    unavailable, CompletionConfig, CompletionResponse, Message, MessageRole, RuntimeAdapter,
    RuntimeAdapterError, RuntimeBackend, RuntimeCapabilities, RuntimeFuture, RuntimeHealth,
    RuntimeHealthFuture, RuntimeStream, StreamChunk,
};

const BACKEND: RuntimeBackend = RuntimeBackend::FreePool;

/// FreePool adapter implementing `RuntimeAdapter`.
pub struct FreePoolAdapter {
    dispatcher: Arc<FreePoolDispatcher>,
    quota: Arc<QuotaTracker>,
    client: reqwest::Client,
    heal_cfg: HealConfig,
}

impl FreePoolAdapter {
    /// Build from a `RuntimeFactoryConfig`. The FreePool adapter does not
    /// require an api_key (keys are collected from env at dispatcher build
    /// time). Returns `Ok` as long as the quota state path can be resolved.
    pub fn from_factory_config(
        cfg: &super::types::RuntimeFactoryConfig,
    ) -> Result<Self, RuntimeAdapterError> {
    let _ = cfg;
        let state_path = default_quota_state_path();
        let limits = std::collections::HashMap::new();
        let quota = QuotaTracker::new(limits, state_path);
        let dispatcher = FreePoolDispatcher::new();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| RuntimeAdapterError::Network(format!("reqwest client build: {e}")))?;

        Ok(Self {
            dispatcher: Arc::new(dispatcher),
            quota: Arc::new(quota),
            client,
            heal_cfg: HealConfig::default(),
        })
    }

    /// Build with explicit keys + quota path (for testing).
    #[cfg(test)]
    fn with_keys_and_path(
        keys: &[(String, String)],
        state_path: PathBuf,
    ) -> Result<Self, RuntimeAdapterError> {
        let quota = QuotaTracker::unlimited(state_path);
        let dispatcher = FreePoolDispatcher::with_keys(keys);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| RuntimeAdapterError::Network(format!("reqwest client build: {e}")))?;
        Ok(Self {
            dispatcher: Arc::new(dispatcher),
            quota: Arc::new(quota),
            client,
            heal_cfg: HealConfig::default(),
        })
    }

    fn build_request_body(
        messages: &[Message],
        config: &CompletionConfig,
        deployment: &Deployment,
    ) -> serde_json::Value {
        let model = config
            .model
            .as_deref()
            .unwrap_or(&deployment.model);

        let msgs: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                json!({
                    "role": m.role.as_str(),
                    "content": m.content,
                })
            })
            .collect();

        let mut body = json!({
            "model": model,
            "messages": msgs,
            "max_tokens": config.max_tokens,
            "temperature": config.temperature,
        });

        if !config.stop_sequences.is_empty() {
            body["stop"] = json!(config.stop_sequences);
        }

        if let Some(seed) = config.seed {
            body["seed"] = json!(seed);
        }

        body
    }

    fn parse_completion_response(
        body: &serde_json::Value,
        deployment: &Deployment,
    ) -> Result<CompletionResponse, RuntimeAdapterError> {
        let text = body
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        let stop_reason = body
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("finish_reason"))
            .and_then(|f| f.as_str())
            .map(|s| s.to_string());

        let usage_input = body
            .get("usage")
            .and_then(|u| u.get("prompt_tokens"))
            .and_then(|t| t.as_u64())
            .unwrap_or(0) as u32;

        let usage_output = body
            .get("usage")
            .and_then(|u| u.get("completion_tokens"))
            .and_then(|t| t.as_u64())
            .unwrap_or(0) as u32;

        let model = body
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or(&deployment.model)
            .to_string();

        Ok(CompletionResponse {
            text,
            stop_reason,
            usage: (usage_input, usage_output),
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
            model,
        })
    }
}

impl RuntimeAdapter for FreePoolAdapter {
    fn backend(&self) -> RuntimeBackend {
        BACKEND
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities {
            supports_tools: false,
            supports_streaming: false,
            supports_vision: false,
            max_context: 32_768,
            max_output: 4_096,
            json_schema_mode: false,
            supports_prefill: false,
            supports_caching: false,
            native_thinking_budget: false,
        }
    }

    fn health_check(&self) -> RuntimeHealthFuture<'_> {
        let pollinations_present = self
            .dispatcher
            .deployments()
            .iter()
            .any(|d| d.keyless);
        Box::pin(async move {
            if pollinations_present {
                RuntimeHealth::Ready
            } else {
                unavailable("free pool has no keyless overflow deployment")
            }
        })
    }

    fn complete(
        &self,
        messages: Vec<Message>,
        config: CompletionConfig,
    ) -> RuntimeFuture<'_, CompletionResponse> {
        let dispatcher = self.dispatcher.clone();
        let quota = self.quota.clone();
        let client = self.client.clone();
        let heal_cfg = self.heal_cfg.clone();

        Box::pin(async move {
            // Cap total HTTP attempts across network/5xx/JSON failures AND
            // 429 re-selections. PoolExhausted heal is bounded inside
            // select_with_heal; this counter bounds the outer request loop so a
            // persistently failing provider cannot hang forever (verifier gate).
            let max_attempts = heal_cfg.max_retries.saturating_add(1).max(3);
            let mut attempt = 0u32;
            loop {
                if attempt >= max_attempts {
                    return Err(RuntimeAdapterError::Other(format!(
                        "free pool gave up after {max_attempts} HTTP attempts"
                    )));
                }
                attempt = attempt.saturating_add(1);

                let deployment = match select_with_heal(&dispatcher, &quota, &heal_cfg).await {
                    Ok(d) => d,
                    Err(SelectError::SpillRequested) => {
                        return Err(RuntimeAdapterError::Other(
                            "free pool exhausted, cloud spill requested".to_string(),
                        ));
                    }
                    Err(SelectError::Exhausted) => {
                        return Err(RuntimeAdapterError::Other(
                            "free pool exhausted, all retries spent".to_string(),
                        ));
                    }
                };

                let api_key = if deployment.keyless {
                    "none".to_string()
                } else {
                    std::env::var(&deployment.env_key_name).unwrap_or_default()
                };

                let body = Self::build_request_body(&messages, &config, &deployment);
                let url = format!("{}/chat/completions", deployment.base_url);

                let resp = client
                    .post(&url)
                    .header("Authorization", format!("Bearer {api_key}"))
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await;

                let resp = match resp {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::warn!(
                            target = "free_pool",
                            "network error on {} ({}): {e}",
                            deployment.provider,
                            deployment.model,
                        );
                        continue;
                    }
                };

                let status = resp.status();

                if status.as_u16() == 429 {
                    let headers: Vec<(String, String)> = resp
                        .headers()
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                        .collect();
                    let cooldown = quota.note_429(&deployment.key_id, &deployment.model, &headers);
                    tracing::debug!(
                        target = "free_pool",
                        "429 on {} ({}), cooldown {}s",
                        deployment.provider,
                        deployment.model,
                        cooldown,
                    );
                    continue;
                }

                if !status.is_success() {
                    tracing::warn!(
                        target = "free_pool",
                        "HTTP {} on {} ({})",
                        status,
                        deployment.provider,
                        deployment.model,
                    );
                    continue;
                }

                let json_body: serde_json::Value = match resp.json().await {
                    Ok(j) => j,
                    Err(e) => {
                        tracing::warn!(
                            target = "free_pool",
                            "JSON parse error on {} ({}): {e}",
                            deployment.provider,
                            deployment.model,
                        );
                        continue;
                    }
                };

                let total_tokens = json_body
                    .get("usage")
                    .and_then(|u| u.get("total_tokens"))
                    .and_then(|t| t.as_u64())
                    .unwrap_or(0);
                quota.record_success(&deployment.key_id, &deployment.model, total_tokens);

                return Self::parse_completion_response(&json_body, &deployment);
            }
        })
    }

    fn stream(
        &self,
        _messages: Vec<Message>,
        _config: CompletionConfig,
    ) -> RuntimeStream<'static> {
        let err = RuntimeAdapterError::NotImplemented {
            backend: BACKEND,
            hint: "free_pool does not support streaming; use complete()",
        };
        Box::pin(stream::once(async move { Err(err) }))
    }
}

fn default_quota_state_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(home).join(".elai").join("free_pool");
    dir.join("quota_state.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_adapter() -> FreePoolAdapter {
        let path = std::env::temp_dir().join(format!(
            "fp_adapter_test_{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        FreePoolAdapter::with_keys_and_path(&[], path).expect("adapter")
    }

    #[test]
    fn backend_is_free_pool() {
        let adapter = tmp_adapter();
        assert_eq!(adapter.backend(), RuntimeBackend::FreePool);
    }

    #[test]
    fn capabilities_no_streaming_no_tools() {
        let adapter = tmp_adapter();
        let caps = adapter.capabilities();
        assert!(!caps.supports_streaming);
        assert!(!caps.supports_tools);
    }

    #[tokio::test]
    async fn health_check_ready_with_pollinations() {
        let adapter = tmp_adapter();
        let health = adapter.health_check().await;
        assert!(matches!(health, RuntimeHealth::Ready));
    }

    #[test]
    fn build_request_body_uses_deployment_model_when_no_config_model() {
        let deployment = Deployment {
            group: ProviderGroup::Big,
            model: "llama-4-scout".to_string(),
            provider: "groq".to_string(),
            base_url: "https://api.groq.com/openai/v1".to_string(),
            key_id: KeyId::from_raw_key("groq", "test"),
            env_key_name: "GROQ_API_KEY".to_string(),
            keyless: false,
        };
        let config = CompletionConfig::default();
        let body = FreePoolAdapter::build_request_body(
            &[Message {
                role: MessageRole::User,
                content: "hello".to_string(),
            }],
            &config,
            &deployment,
        );
        assert_eq!(body["model"], "llama-4-scout");
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["messages"][0]["content"], "hello");
    }

    #[test]
    fn parse_completion_response_extracts_text() {
        let deployment = Deployment {
            group: ProviderGroup::Big,
            model: "llama-4-scout".to_string(),
            provider: "groq".to_string(),
            base_url: "https://api.groq.com/openai/v1".to_string(),
            key_id: KeyId::from_raw_key("groq", "test"),
            env_key_name: "GROQ_API_KEY".to_string(),
            keyless: false,
        };
        let body = json!({
            "model": "llama-4-scout",
            "choices": [{
                "message": { "role": "assistant", "content": "Hello!" },
                "finish_reason": "stop",
                "index": 0
            }],
            "usage": { "prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15 }
        });
        let resp = FreePoolAdapter::parse_completion_response(&body, &deployment).expect("parse");
        assert_eq!(resp.text, "Hello!");
        assert_eq!(resp.stop_reason, Some("stop".to_string()));
        assert_eq!(resp.usage, (10, 5));
        assert_eq!(resp.model, "llama-4-scout");
    }

    #[test]
    fn parse_completion_response_empty_body_returns_empty_text() {
        let deployment = Deployment {
            group: ProviderGroup::OpenrouterFree,
            model: "openai".to_string(),
            provider: "pollinations".to_string(),
            base_url: "https://text.pollinations.ai/openai".to_string(),
            key_id: KeyId::from_raw_key("pollinations", "none"),
            env_key_name: "none".to_string(),
            keyless: true,
        };
        let body = json!({});
        let resp = FreePoolAdapter::parse_completion_response(&body, &deployment).expect("parse");
        assert_eq!(resp.text, "");
        assert_eq!(resp.usage, (0, 0));
    }
}
