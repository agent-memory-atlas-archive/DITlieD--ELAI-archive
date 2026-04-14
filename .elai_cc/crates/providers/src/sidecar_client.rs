//! Inference sidecar client — connects to the ELAI Python inference server.
//!
//! The sidecar server (python/elai_inference/server.py) runs HF Transformers
//! with DoLa + TurboQuant composed. This client speaks the same OpenAI-compatible
//! protocol as VllmProvider, so the orchestrator doesn't need to know the difference.
//!
//! All code in this module is gated behind `cfg(feature = "sidecar")`.

use std::sync::{Arc, Mutex};
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

fn default_etc_threshold() -> f64 {
    f64::INFINITY
}

/// Serde helper: serialize f64::INFINITY as null, deserialize null/missing as infinity.
mod etc_threshold_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if value.is_infinite() {
            serializer.serialize_none()
        } else {
            serializer.serialize_f64(*value)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<f64> = Option::deserialize(deserializer)?;
        Ok(opt.unwrap_or(f64::INFINITY))
    }
}

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
    /// Whether steering vectors are enabled.
    pub steering_enabled: bool,
    /// Steering coefficient (typical range [3, 20]).
    pub steering_coefficient: f64,
    /// Steering target layer (or "auto" for model_layers / 3).
    pub steering_layer: String,
    /// Temperature for generation.
    pub temperature: f32,
    /// Max tokens per completion.
    pub max_tokens: u32,
    /// ETC entropy threshold (f64::INFINITY = disabled until calibration).
    #[serde(default = "default_etc_threshold", with = "etc_threshold_serde")]
    pub etc_threshold: f64,
}

impl Default for SidecarConfig {
    fn default() -> Self {
        Self {
            server_url: DEFAULT_SIDECAR_URL.to_string(),
            model: String::new(),
            dola_enabled: true,
            dola_alpha: 0.1,
            turboquant_bits: 4,
            steering_enabled: false,
            steering_coefficient: 5.0,
            steering_layer: "auto".to_string(),
            temperature: 0.0,
            max_tokens: 256,
            etc_threshold: f64::INFINITY,
        }
    }
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// ETC (Entropy-Trend Constraint) search trigger payload.
///
/// Populated when ETC detects a high-entropy moment in import context during
/// generation. The orchestrator can use this to inject search results and
/// regenerate from the trigger point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchTriggerPayload {
    /// Generated text up to the high-entropy trigger point.
    pub partial_text: String,
    /// What context triggered ETC (e.g., "import").
    pub detected_context: String,
    /// Token index where the trigger occurred.
    pub trigger_token_idx: i64,
}

#[derive(Debug, Clone, Deserialize)]
struct CompletionChoice {
    text: String,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CompletionResponse {
    choices: Vec<CompletionChoice>,
    /// ETC search trigger — present only when entropy monitoring fires.
    search_trigger: Option<SearchTriggerPayload>,
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
    /// Last ETC search trigger from the most recent completion (if any).
    last_search_trigger: Mutex<Option<SearchTriggerPayload>>,
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
            last_search_trigger: Mutex::new(None),
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
    ///
    /// Returns (completions, optional ETC search trigger).
    async fn complete_impl(
        &self,
        prompt: &str,
    ) -> Result<(Vec<String>, Option<SearchTriggerPayload>), ApiError> {
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
        Ok((tokens, resp.search_trigger))
    }

    /// Access the last ETC search trigger from the most recent completion.
    /// Callers should check this after `complete()` to decide on search injection.
    pub fn last_search_trigger(&self) -> Option<SearchTriggerPayload> {
        self.last_search_trigger
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
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

            match result {
                Ok((completions, trigger)) => {
                    let triggered = trigger.is_some();
                    if let Ok(mut guard) = self.last_search_trigger.lock() {
                        *guard = trigger;
                    }
                    tracing::debug!(
                        target: "providers::sidecar",
                        elapsed_ms = elapsed.as_millis() as u64,
                        dola = self.config.dola_enabled,
                        turboquant_bits = self.config.turboquant_bits,
                        etc_triggered = triggered,
                        "sidecar completion finished"
                    );
                    Ok(completions)
                }
                Err(e) => {
                    tracing::debug!(
                        target: "providers::sidecar",
                        elapsed_ms = elapsed.as_millis() as u64,
                        error = %e,
                        "sidecar completion failed"
                    );
                    Err(e)
                }
            }
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
        assert!(config.etc_threshold.is_infinite());
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
            steering_enabled: true,
            steering_coefficient: 5.0,
            steering_layer: "auto".to_string(),
            temperature: 0.7,
            max_tokens: 512,
            etc_threshold: 2.5,
        };

        let json = serde_json::to_string(&config).expect("serialize");
        let back: SidecarConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(back.dola_enabled);
        assert_eq!(back.turboquant_bits, 3);
        assert_eq!(back.dola_alpha, 0.05);
        assert!((back.etc_threshold - 2.5).abs() < f64::EPSILON);

        // Verify infinity round-trips via null
        let default_config = SidecarConfig::default();
        let json2 = serde_json::to_string(&default_config).expect("serialize default");
        assert!(json2.contains("\"etc_threshold\":null"));
        let back2: SidecarConfig = serde_json::from_str(&json2).expect("deserialize default");
        assert!(back2.etc_threshold.is_infinite());
    }

    #[test]
    fn search_trigger_payload_deserialization() {
        let json = r#"{"partial_text":"import ","detected_context":"import","trigger_token_idx":42}"#;
        let payload: SearchTriggerPayload = serde_json::from_str(json).expect("deserialize");
        assert_eq!(payload.partial_text, "import ");
        assert_eq!(payload.detected_context, "import");
        assert_eq!(payload.trigger_token_idx, 42);
    }
}
