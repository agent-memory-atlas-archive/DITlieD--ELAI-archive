//! vLLM HTTP client — VllmProvider implementing LlmProvider trait.
//!
//! All code in this module is gated behind `cfg(feature = "vllm")`.
//! VllmProvider stores its own VllmConfig and ignores the &OllamaConfig
//! parameter in LlmProvider::complete (impedance mismatch Option A —
//! see workflow/reasoning/phase-3J-decisions.md).

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use elai_core::ClockProvider;

use crate::error::ApiError;
use crate::providers::ollama::{LlmFuture, LlmProvider, OllamaConfig};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Default vLLM server URL (WSL2 port-forwarded).
pub const DEFAULT_VLLM_BASE_URL: &str = "http://localhost:8000";

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const HEALTH_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum retry attempts for retryable (5xx / connect) errors.
const MAX_RETRIES: u32 = 3;
/// Base delay for exponential backoff (doubled each retry, plus jitter).
const BASE_RETRY_DELAY: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VllmConfig {
    pub server_url: String,
    pub model: String,
    pub tensor_parallelism: bool,
    pub served_model_names: Vec<String>,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for VllmConfig {
    fn default() -> Self {
        Self {
            server_url: DEFAULT_VLLM_BASE_URL.to_string(),
            model: String::new(),
            tensor_parallelism: false,
            served_model_names: Vec::new(),
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
}

// ---------------------------------------------------------------------------
// InferenceBackend — enum dispatch for ModelPool (P3J.2)
// ---------------------------------------------------------------------------

/// Backend selection for GPU resource management.
/// When Vllm is active, host-side NVML/VRAM queries are skipped because
/// vLLM manages its own GPU memory via CUDA internally.
/// Sidecar runs HF Transformers with DoLa + TurboQuant composed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InferenceBackend {
    Ollama(OllamaConfig),
    Vllm(VllmConfig),
    #[cfg(feature = "sidecar")]
    Sidecar(crate::sidecar_client::SidecarConfig),
}

// ---------------------------------------------------------------------------
// Health check types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VllmHealthStatus {
    Healthy,
    Unhealthy { reason: String },
}

/// Single model entry from /v1/models response.
#[derive(Debug, Clone, Deserialize)]
pub struct VllmModelEntry {
    pub id: String,
    pub object: String,
}

/// /v1/models response envelope.
#[derive(Debug, Clone, Deserialize)]
pub struct VllmModelsResponse {
    pub data: Vec<VllmModelEntry>,
}

// ---------------------------------------------------------------------------
// vLLM SSE response types (OpenAI-compatible format)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct VllmCompletionChunk {
    pub choices: Vec<VllmChoice>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VllmChoice {
    pub index: u32,
    pub text: Option<String>,
    pub delta: Option<VllmDelta>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VllmDelta {
    pub content: Option<String>,
}

// ---------------------------------------------------------------------------
// VllmProvider
// ---------------------------------------------------------------------------

pub struct VllmProvider {
    config: VllmConfig,
    client: reqwest::Client,
    clock: Arc<dyn ClockProvider>,
}

impl std::fmt::Debug for VllmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VllmProvider")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl VllmProvider {
    /// Create a new VllmProvider.
    ///
    /// # Errors
    /// Returns `ApiError::Http` if the reqwest client cannot be built.
    pub fn new(config: VllmConfig, clock: Arc<dyn ClockProvider>) -> Result<Self, ApiError> {
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

    /// Check vLLM server health via GET /health.
    pub async fn health(&self) -> Result<VllmHealthStatus, ApiError> {
        let url = format!("{}/health", self.config.server_url);
        let result = tokio::time::timeout(HEALTH_TIMEOUT, self.client.get(&url).send()).await;

        match result {
            Ok(Ok(response)) if response.status().is_success() => Ok(VllmHealthStatus::Healthy),
            Ok(Ok(response)) => Ok(VllmHealthStatus::Unhealthy {
                reason: format!("HTTP {}", response.status()),
            }),
            Ok(Err(e)) => Ok(VllmHealthStatus::Unhealthy {
                reason: e.to_string(),
            }),
            Err(_) => Ok(VllmHealthStatus::Unhealthy {
                reason: "health check timed out after 5s".to_string(),
            }),
        }
    }

    /// List available models via GET /v1/models.
    pub async fn list_models(&self) -> Result<VllmModelsResponse, ApiError> {
        let url = format!("{}/v1/models", self.config.server_url);
        let response = tokio::time::timeout(HEALTH_TIMEOUT, self.client.get(&url).send())
            .await
            .map_err(|_| ApiError::Api {
                status: reqwest::StatusCode::REQUEST_TIMEOUT,
                error_type: Some("timeout".to_string()),
                message: Some("list_models timed out after 5s".to_string()),
                body: String::new(),
                retryable: true,
            })?
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

        let models: VllmModelsResponse = response.json().await.map_err(ApiError::Http)?;
        Ok(models)
    }

    /// Send a completion request to /v1/completions with SSE streaming.
    /// Collects all tokens and returns them as a Vec<String>.
    async fn complete_impl(&self, prompt: &str) -> Result<Vec<String>, ApiError> {
        let url = format!("{}/v1/completions", self.config.server_url);
        let body = serde_json::json!({
            "model": &self.config.model,
            "prompt": prompt,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "stream": true,
        });

        let response = tokio::time::timeout(
            REQUEST_TIMEOUT,
            self.client.post(&url).json(&body).send(),
        )
        .await
        .map_err(|_| ApiError::Api {
            status: reqwest::StatusCode::REQUEST_TIMEOUT,
            error_type: Some("timeout".to_string()),
            message: Some("completion request timed out after 30s".to_string()),
            body: String::new(),
            retryable: true,
        })?
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

        // Parse SSE stream
        let full_body = response.text().await.map_err(ApiError::Http)?;
        let tokens = parse_vllm_sse_text(&full_body);
        Ok(tokens)
    }

    /// Send a completion request with retries (exponential backoff, no retry on 4xx).
    async fn complete_with_retry(&self, prompt: &str) -> Result<Vec<String>, ApiError> {
        let mut last_error: Option<ApiError> = None;

        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                if let Some(ref err) = last_error {
                    if !err.is_retryable() {
                        // Safety: last_error is confirmed Some by the if-let above
                        if let Some(e) = last_error.take() {
                            return Err(e);
                        }
                    }
                }
                let delay = backoff_delay(attempt, BASE_RETRY_DELAY)?;
                tokio::time::sleep(delay).await;
            }

            match self.complete_impl(prompt).await {
                Ok(tokens) => return Ok(tokens),
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        match last_error {
            Some(e) => Err(ApiError::RetriesExhausted {
                attempts: MAX_RETRIES + 1,
                last_error: Box::new(e),
            }),
            None => Err(ApiError::RetriesExhausted {
                attempts: MAX_RETRIES + 1,
                last_error: Box::new(ApiError::Api {
                    status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                    error_type: Some("internal".to_string()),
                    message: Some("retry loop completed without error or success".to_string()),
                    body: String::new(),
                    retryable: false,
                }),
            }),
        }
    }

    /// Send an embedding request to /v1/embeddings.
    async fn embed_impl(&self, text: &str) -> Result<Vec<f32>, ApiError> {
        let url = format!("{}/v1/embeddings", self.config.server_url);
        let body = serde_json::json!({
            "model": &self.config.model,
            "input": text,
        });

        let response = tokio::time::timeout(
            REQUEST_TIMEOUT,
            self.client.post(&url).json(&body).send(),
        )
        .await
        .map_err(|_| ApiError::Api {
            status: reqwest::StatusCode::REQUEST_TIMEOUT,
            error_type: Some("timeout".to_string()),
            message: Some("embedding request timed out after 30s".to_string()),
            body: String::new(),
            retryable: true,
        })?
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

        let resp: serde_json::Value = response.json().await.map_err(ApiError::Http)?;
        // OpenAI embeddings response: {"data": [{"embedding": [f32...]}]}
        let embedding = resp
            .get("data")
            .and_then(|d| d.get(0))
            .and_then(|entry| entry.get("embedding"))
            .and_then(|e| serde_json::from_value::<Vec<f32>>(e.clone()).ok())
            .ok_or_else(|| ApiError::Api {
                status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                error_type: Some("parse_error".to_string()),
                message: Some("failed to parse embedding response".to_string()),
                body: String::new(),
                retryable: false,
            })?;

        Ok(embedding)
    }
}

// ---------------------------------------------------------------------------
// LlmProvider trait implementation
// ---------------------------------------------------------------------------

impl LlmProvider for VllmProvider {
    fn complete<'a>(
        &'a self,
        prompt: &'a str,
        _config: &'a OllamaConfig, // Ignored — VllmProvider uses self.config (Option A)
    ) -> LlmFuture<'a, Vec<String>> {
        Box::pin(async move {
            let start = self.clock.now();
            let result = self.complete_with_retry(prompt).await;
            let elapsed = self.clock.elapsed_since(start);
            tracing::debug!(
                target: "providers::vllm",
                elapsed_ms = elapsed.as_millis() as u64,
                "completion request finished"
            );
            result
        })
    }

    fn embed<'a>(&'a self, text: &'a str) -> LlmFuture<'a, Vec<f32>> {
        Box::pin(async move { self.embed_impl(text).await })
    }

    fn token_count(&self, text: &str) -> Result<usize, ApiError> {
        // Rough estimate: ~4 chars per token (same heuristic as OllamaProvider).
        // Accurate tokenization requires a tokenizer library or server roundtrip.
        Ok(text.len() / 4 + 1)
    }

    fn token_count_estimate(&self, text: &str) -> usize {
        text.len() / 4 + 1
    }
}

// ---------------------------------------------------------------------------
// VllmModelPool — ModelPool impl for vLLM backend (P3J.2)
// ---------------------------------------------------------------------------

/// ModelPool implementation when using vLLM as the inference backend.
/// Returns u64::MAX for available_vram because vLLM manages GPU memory
/// internally — no host-side NVML queries needed.
pub struct VllmModelPool {
    backend: InferenceBackend,
}

impl VllmModelPool {
    #[must_use]
    pub fn new(backend: InferenceBackend) -> Self {
        Self { backend }
    }

    /// Returns the inference backend reference.
    #[must_use]
    pub fn backend(&self) -> &InferenceBackend {
        &self.backend
    }

    /// Check if using vLLM backend. When true, NVML/VRAM queries are skipped.
    #[must_use]
    pub fn is_vllm(&self) -> bool {
        matches!(self.backend, InferenceBackend::Vllm(_))
    }

    /// Check if using sidecar backend (DoLa + TurboQuant composed).
    #[must_use]
    pub fn is_sidecar(&self) -> bool {
        #[cfg(feature = "sidecar")]
        {
            matches!(self.backend, InferenceBackend::Sidecar(_))
        }
        #[cfg(not(feature = "sidecar"))]
        {
            false
        }
    }
}

// We implement the ModelPool trait from orchestrator.
// However, since providers should not depend on orchestrator,
// we provide the same interface via a standalone method.
// The orchestrator can wrap this in an adapter if needed.
impl VllmModelPool {
    /// Available VRAM in bytes.
    /// For Vllm backend: u64::MAX (vLLM manages GPU internally).
    /// For Ollama backend: u64::MAX (no NVML — real impl deferred to GPU crate).
    /// For Sidecar backend: u64::MAX (HF Transformers manages GPU internally).
    #[must_use]
    pub fn available_vram(&self) -> u64 {
        // All backends currently return u64::MAX — real NVML deferred
        u64::MAX
    }
}

// ---------------------------------------------------------------------------
// SSE parsing helpers (vLLM OpenAI-format)
// ---------------------------------------------------------------------------

/// Parse vLLM SSE text response into individual tokens.
/// vLLM SSE format: lines like `data: {"choices":[{"text":"token",...}]}`
/// Terminated by `data: [DONE]`.
fn parse_vllm_sse_text(body: &str) -> Vec<String> {
    let mut tokens = Vec::new();

    for line in body.lines() {
        let data = match line.strip_prefix("data:") {
            Some(d) => d.trim(),
            None => continue,
        };

        if data == "[DONE]" || data.is_empty() {
            continue;
        }

        if let Ok(chunk) = serde_json::from_str::<VllmCompletionChunk>(data) {
            for choice in &chunk.choices {
                // /v1/completions uses "text" field
                if let Some(ref text) = choice.text {
                    if !text.is_empty() {
                        tokens.push(text.clone());
                    }
                }
                // /v1/chat/completions uses "delta.content" field
                if let Some(ref delta) = choice.delta {
                    if let Some(ref content) = delta.content {
                        if !content.is_empty() {
                            tokens.push(content.clone());
                        }
                    }
                }
            }
        }
    }

    tokens
}

/// Compute exponential backoff delay with jitter.
/// Never retries 4xx (caller checks is_retryable before calling).
fn backoff_delay(attempt: u32, base: Duration) -> Result<Duration, ApiError> {
    let shift = attempt.saturating_sub(1);
    if shift >= 32 {
        return Err(ApiError::BackoffOverflow {
            attempt,
            base_delay: base,
        });
    }
    let multiplier = 1u64 << shift;
    let base_ms = base.as_millis() as u64;
    let delay_ms = base_ms.checked_mul(multiplier).ok_or(ApiError::BackoffOverflow {
        attempt,
        base_delay: base,
    })?;
    // Simple jitter: add 0-25% of delay
    let jitter_ms = delay_ms / 4;
    Ok(Duration::from_millis(delay_ms.saturating_add(jitter_ms)))
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::OsClock;

    // -----------------------------------------------------------------------
    // P3J.1 Tests
    // -----------------------------------------------------------------------

    #[test]
    fn vllm_config_default_has_sane_values() {
        let config = VllmConfig::default();
        assert_eq!(config.server_url, "http://localhost:8000");
        assert!(!config.tensor_parallelism);
        assert!(config.served_model_names.is_empty());
        assert!(config.temperature > 0.0);
        assert!(config.max_tokens > 0);
    }

    #[test]
    fn vllm_provider_creation_succeeds() {
        let config = VllmConfig::default();
        let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
        let provider = VllmProvider::new(config, clock);
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn vllm_provider_timeout_on_unreachable_server() {
        // Point at a definitely-unreachable address
        let config = VllmConfig {
            server_url: "http://192.0.2.1:19999".to_string(), // TEST-NET, guaranteed unreachable
            model: "test-model".to_string(),
            ..VllmConfig::default()
        };
        let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
        let provider = VllmProvider::new(config, clock).expect("client build should succeed");

        let start = std::time::Instant::now();
        let result = provider.health().await;
        let elapsed = start.elapsed();

        // Should return Unhealthy, NOT hang
        match result {
            Ok(VllmHealthStatus::Unhealthy { reason }) => {
                assert!(
                    !reason.is_empty(),
                    "unhealthy reason should describe the failure"
                );
            }
            other => panic!("expected Unhealthy, got {other:?}"),
        }
        // Must complete within 10s (5s timeout + margin)
        assert!(
            elapsed < Duration::from_secs(10),
            "health check took {elapsed:?}, expected < 10s"
        );
    }

    #[tokio::test]
    async fn vllm_provider_complete_returns_error_on_unreachable() {
        let config = VllmConfig {
            server_url: "http://192.0.2.1:19999".to_string(),
            model: "test-model".to_string(),
            max_tokens: 10,
            ..VllmConfig::default()
        };
        let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
        let provider = VllmProvider::new(config, clock).expect("client build should succeed");
        let ollama_config = OllamaConfig::default();

        let result = provider.complete("hello", &ollama_config).await;
        assert!(result.is_err(), "should fail on unreachable server");
    }

    #[test]
    fn vllm_sse_parsing_extracts_tokens() {
        let sse_body = concat!(
            "data: {\"choices\":[{\"index\":0,\"text\":\"Hello\",\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"text\":\" world\",\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"text\":\"!\",\"finish_reason\":\"stop\"}]}\n\n",
            "data: [DONE]\n\n",
        );

        let tokens = parse_vllm_sse_text(sse_body);
        assert_eq!(tokens, vec!["Hello", " world", "!"]);
    }

    #[test]
    fn vllm_sse_parsing_handles_chat_format() {
        let sse_body = concat!(
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hi\"},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\" there\"},\"finish_reason\":null}]}\n\n",
            "data: [DONE]\n\n",
        );

        let tokens = parse_vllm_sse_text(sse_body);
        assert_eq!(tokens, vec!["Hi", " there"]);
    }

    #[test]
    fn vllm_sse_parsing_ignores_empty_and_done() {
        let sse_body = "data: [DONE]\n\n";
        let tokens = parse_vllm_sse_text(sse_body);
        assert!(tokens.is_empty());
    }

    #[test]
    fn vllm_sse_parsing_handles_malformed_json_gracefully() {
        let sse_body = concat!(
            "data: {not valid json}\n\n",
            "data: {\"choices\":[{\"index\":0,\"text\":\"ok\",\"finish_reason\":null}]}\n\n",
            "data: [DONE]\n\n",
        );

        let tokens = parse_vllm_sse_text(sse_body);
        assert_eq!(tokens, vec!["ok"]); // Malformed lines skipped, valid ones parsed
    }

    #[test]
    fn vllm_health_status_variants() {
        let healthy = VllmHealthStatus::Healthy;
        let unhealthy = VllmHealthStatus::Unhealthy {
            reason: "down".to_string(),
        };
        assert_eq!(healthy, VllmHealthStatus::Healthy);
        assert_ne!(healthy, unhealthy);
    }

    #[test]
    fn token_count_estimate_matches_heuristic() {
        let config = VllmConfig {
            model: "test".to_string(),
            ..VllmConfig::default()
        };
        let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
        let provider = VllmProvider::new(config, clock).expect("should build");

        // "hello world" = 11 chars => 11/4 + 1 = 3
        assert_eq!(provider.token_count_estimate("hello world"), 3);
        assert_eq!(
            provider.token_count("hello world").expect("should succeed"),
            3
        );
    }

    #[test]
    fn backoff_delay_increases_exponentially() {
        let d1 = backoff_delay(1, BASE_RETRY_DELAY).expect("should compute");
        let d2 = backoff_delay(2, BASE_RETRY_DELAY).expect("should compute");
        let d3 = backoff_delay(3, BASE_RETRY_DELAY).expect("should compute");

        assert!(d2 > d1, "d2={d2:?} should be > d1={d1:?}");
        assert!(d3 > d2, "d3={d3:?} should be > d2={d2:?}");
    }

    #[test]
    fn backoff_delay_handles_overflow() {
        let result = backoff_delay(64, BASE_RETRY_DELAY);
        assert!(result.is_err(), "extreme attempt should overflow");
    }

    // -----------------------------------------------------------------------
    // P3J.2 Tests
    // -----------------------------------------------------------------------

    #[test]
    fn inference_backend_enum_roundtrip() {
        let vllm = InferenceBackend::Vllm(VllmConfig::default());
        let json = serde_json::to_string(&vllm).expect("serialize");
        let back: InferenceBackend = serde_json::from_str(&json).expect("deserialize");
        match back {
            InferenceBackend::Vllm(c) => {
                assert_eq!(c.server_url, DEFAULT_VLLM_BASE_URL);
            }
            InferenceBackend::Ollama(_) => panic!("expected Vllm variant"),
        }
    }

    #[test]
    fn inference_backend_ollama_roundtrip() {
        let ollama = InferenceBackend::Ollama(OllamaConfig::default());
        let json = serde_json::to_string(&ollama).expect("serialize");
        let back: InferenceBackend = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(back, InferenceBackend::Ollama(_)));
    }

    #[test]
    fn modelpool_vllm_backend_no_nvml() {
        let pool = VllmModelPool::new(InferenceBackend::Vllm(VllmConfig::default()));
        assert!(pool.is_vllm());
        // u64::MAX = vLLM manages its own GPU, no NVML queries
        assert_eq!(pool.available_vram(), u64::MAX);
    }

    #[test]
    fn modelpool_ollama_backend() {
        let pool = VllmModelPool::new(InferenceBackend::Ollama(OllamaConfig::default()));
        assert!(!pool.is_vllm());
        assert_eq!(pool.available_vram(), u64::MAX);
    }

    #[test]
    fn vllm_config_serialization() {
        let config = VllmConfig {
            server_url: "http://localhost:8000".to_string(),
            model: "gemma4:e2b".to_string(),
            tensor_parallelism: true,
            served_model_names: vec!["gemma4".to_string()],
            temperature: 0.5,
            max_tokens: 1024,
        };

        let json = serde_json::to_string(&config).expect("serialize");
        assert!(json.contains("tensor_parallelism"));
        assert!(json.contains("gemma4:e2b"));

        let back: VllmConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(back.tensor_parallelism);
        assert_eq!(back.model, "gemma4:e2b");
    }
}
