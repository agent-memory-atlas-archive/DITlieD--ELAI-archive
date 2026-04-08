//! Inference sidecar client — connects to the ELAI Python inference server.
//!
//! The sidecar server (python/elai_inference/server.py) runs HF Transformers
//! with DoLa + TurboQuant composed. This client speaks the same OpenAI-compatible
//! protocol as VllmProvider, so the orchestrator doesn't need to know the difference.
//!
//! All code in this module is gated behind `cfg(feature = "sidecar")`.

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use elai_core::ClockProvider;

use crate::error::ApiError;
use crate::providers::ollama::{LlmFuture, LlmProvider, OllamaConfig};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Default sidecar server URL.
pub const DEFAULT_SIDECAR_URL: &str = "http://localhost:8100";

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120); // Longer — HF Transformers is slower

/// Sidecar inference configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarConfig {
    /// Server URL (e.g., "http://localhost:8100").
    pub server_url: String,
    /// Model name (informational — the sidecar loads the model at startup).
    pub model: String,
    /// Whether DoLa is enabled on the sidecar.
    pub dola_enabled: bool,
    /// DoLa alpha parameter.
    pub dola_alpha: f64,
    /// TurboQuant bit-width (0 = disabled).
    pub turboquant_bits: u8,
    /// Temperature for generation.
    pub temperature: f32,
    /// Max tokens per completion.
    pub max_tokens: u32,
}

impl Default for SidecarConfig {
    fn default() -> Self {
        Self {
            server_url: DEFAULT_SIDECAR_URL.to_string(),
            model: String::new(),
            dola_enabled: true,
            dola_alpha: 0.1,
            turboquant_bits: 4,
            temperature: 0.0,
            max_tokens: 256,
        }
    }
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
struct CompletionChoice {
    text: String,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CompletionResponse {
    choices: Vec<CompletionChoice>,
}

// ---------------------------------------------------------------------------
// SidecarProvider
// ---------------------------------------------------------------------------

/// Provider that connects to the ELAI inference sidecar.
/// The sidecar handles DoLa + TurboQuant internally.
pub struct SidecarProvider {
    config: SidecarConfig,
    client: reqwest::Client,
    clock: Arc<dyn ClockProvider>,
}

impl std::fmt::Debug for SidecarProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SidecarProvider")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl SidecarProvider {
    /// Create a new SidecarProvider.
    ///
    /// # Errors
    /// Returns `ApiError::Http` if the reqwest client cannot be built.
    pub fn new(config: SidecarConfig, clock: Arc<dyn ClockProvider>) -> Result<Self, ApiError> {
        let client = reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()?;
        Ok(Self {
            config,
            client,
            clock,
        })
    }

    /// Check sidecar health via GET /health.
    pub async fn health(&self) -> Result<bool, ApiError> {
        let url = format!("{}/health", self.config.server_url);
        let result = tokio::time::timeout(
            Duration::from_secs(5),
            self.client.get(&url).send(),
        )
        .await;

        match result {
            Ok(Ok(response)) if response.status().is_success() => Ok(true),
            _ => Ok(false),
        }
    }

    /// Send a completion request to /v1/completions.
    async fn complete_impl(&self, prompt: &str) -> Result<Vec<String>, ApiError> {
        let url = format!("{}/v1/completions", self.config.server_url);
        let body = serde_json::json!({
            "prompt": prompt,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(ApiError::Http)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<unreadable>".to_string());
            return Err(ApiError::Api {
                status,
                error_type: None,
                message: None,
                body,
                retryable: status.is_server_error(),
            });
        }

        let resp: CompletionResponse = response.json().await.map_err(ApiError::Http)?;
        let tokens: Vec<String> = resp
            .choices
            .into_iter()
            .filter_map(|c| {
                if c.text.is_empty() {
                    None
                } else {
                    Some(c.text)
                }
            })
            .collect();
        Ok(tokens)
    }
}

// ---------------------------------------------------------------------------
// LlmProvider trait implementation
// ---------------------------------------------------------------------------

impl LlmProvider for SidecarProvider {
    fn complete<'a>(
        &'a self,
        prompt: &'a str,
        _config: &'a OllamaConfig,
    ) -> LlmFuture<'a, Vec<String>> {
        Box::pin(async move {
            let start = self.clock.now();
            let result = self.complete_impl(prompt).await;
            let elapsed = self.clock.elapsed_since(start);
            tracing::debug!(
                target: "providers::sidecar",
                elapsed_ms = elapsed.as_millis() as u64,
                dola = self.config.dola_enabled,
                turboquant_bits = self.config.turboquant_bits,
                "sidecar completion finished"
            );
            result
        })
    }

    fn embed<'a>(&'a self, _text: &'a str) -> LlmFuture<'a, Vec<f32>> {
        Box::pin(async move {
            Err(ApiError::Api {
                status: reqwest::StatusCode::NOT_IMPLEMENTED,
                error_type: Some("not_implemented".to_string()),
                message: Some("sidecar does not support embeddings".to_string()),
                body: String::new(),
                retryable: false,
            })
        })
    }

    fn token_count(&self, text: &str) -> Result<usize, ApiError> {
        Ok(text.len() / 4 + 1)
    }

    fn token_count_estimate(&self, text: &str) -> usize {
        text.len() / 4 + 1
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::OsClock;

    #[test]
    fn sidecar_config_defaults() {
        let config = SidecarConfig::default();
        assert_eq!(config.server_url, DEFAULT_SIDECAR_URL);
        assert!(config.dola_enabled);
        assert_eq!(config.turboquant_bits, 4);
        assert_eq!(config.dola_alpha, 0.1);
    }

    #[test]
    fn sidecar_provider_creation() {
        let config = SidecarConfig::default();
        let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
        let provider = SidecarProvider::new(config, clock);
        assert!(provider.is_ok());
    }

    #[test]
    fn sidecar_config_serialization() {
        let config = SidecarConfig {
            server_url: "http://localhost:9000".to_string(),
            model: "gemma4:e2b".to_string(),
            dola_enabled: true,
            dola_alpha: 0.05,
            turboquant_bits: 3,
            temperature: 0.7,
            max_tokens: 512,
        };

        let json = serde_json::to_string(&config).expect("serialize");
        let back: SidecarConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(back.dola_enabled);
        assert_eq!(back.turboquant_bits, 3);
        assert_eq!(back.dola_alpha, 0.05);
    }
}
