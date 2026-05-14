use rustc_hash::FxHashMap;
use std::future::Future;
use std::pin::Pin;

use serde::{Deserialize, Serialize};

use crate::error::ApiError;

pub const DEFAULT_OLLAMA_BASE_URL: &str = "http://localhost:11434";

// ---------------------------------------------------------------------------
// Speculative decoding API probe (Phase 1F.2)
// ---------------------------------------------------------------------------

/// Result of probing whether the local ollama instance supports speculative
/// decoding via a `draft_model` parameter in `/api/generate`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProbeResult {
    /// Ollama accepted and acted on the draft_model parameter.
    Supported {
        ollama_version: String,
        api_params: Vec<String>,
        draft_model: String,
    },
    /// Ollama does NOT support speculative decoding (or silently ignores it).
    NotSupported {
        ollama_version: String,
        reason: String,
    },
}

impl std::fmt::Display for ProbeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Supported {
                ollama_version,
                api_params,
                draft_model,
            } => write!(
                f,
                "SUPPORTED: ollama {ollama_version}, params=[{}], draft_model={draft_model}",
                api_params.join(", ")
            ),
            Self::NotSupported {
                ollama_version,
                reason,
            } => write!(
                f,
                "NOT_SUPPORTED: ollama {ollama_version}, reason={reason}"
            ),
        }
    }
}

/// Low-level response structs for the ollama API (only fields we inspect).
#[derive(Debug, Deserialize)]
struct OllamaVersionResponse {
    version: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields deserialized from API; not all read directly
struct OllamaGenerateResponse {
    response: String,
    #[serde(default)]
    model: String,
    #[serde(default)]
    created_at: String,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    done_reason: String,
    #[serde(default)]
    context: Vec<u64>,
    #[serde(default)]
    total_duration: u64,
    #[serde(default)]
    load_duration: u64,
    #[serde(default)]
    prompt_eval_count: u64,
    #[serde(default)]
    prompt_eval_duration: u64,
    #[serde(default)]
    eval_count: u64,
    #[serde(default)]
    eval_duration: u64,
    /// Captures ONLY unknown keys (draft-related fields if spec decode is supported).
    #[serde(flatten)]
    extra: FxHashMap<String, serde_json::Value>,
}

/// Probe the local ollama instance for speculative decoding support.
///
/// Strategy (addresses adversarial case — ollama silently ignores unknown
/// options):
///   1. GET /api/version  -> record version string.
///   2. POST /api/generate baseline (no draft param, fixed seed, temp=0).
///   3. POST /api/generate with `draft_model` in options.
///   4. Compare: if response contains draft-specific keys (e.g.
///      `draft_tokens_accepted`) OR timing profile is structurally different,
///      feature is active. Otherwise it was silently ignored -> NOT_SUPPORTED.
pub async fn probe_speculative_support(
    base_url: &str,
    model: &str,
    draft_model_candidate: &str,
    timeout: std::time::Duration,
) -> Result<ProbeResult, ApiError> {
    let client = reqwest::Client::builder()
        .timeout(timeout)
        .build()?;

    // 1. Version
    let version_url = format!("{base_url}/api/version");
    let version_resp: OllamaVersionResponse = client
        .get(&version_url)
        .send()
        .await?
        .error_for_status()
        .map_err(ApiError::from)?
        .json()
        .await?;
    let ollama_version = version_resp.version;

    // 2. Baseline generate (no draft, deterministic)
    let baseline_body = serde_json::json!({
        "model": model,
        "prompt": "Count from 1 to 5.",
        "stream": false,
        "options": {
            "seed": 42,
            "temperature": 0.0,
            "num_predict": 20
        }
    });
    let baseline_resp: OllamaGenerateResponse = client
        .post(&format!("{base_url}/api/generate"))
        .json(&baseline_body)
        .send()
        .await?
        .error_for_status()
        .map_err(ApiError::from)?
        .json()
        .await?;

    // 3. Generate WITH draft_model in options
    let draft_body = serde_json::json!({
        "model": model,
        "prompt": "Count from 1 to 5.",
        "stream": false,
        "options": {
            "seed": 42,
            "temperature": 0.0,
            "num_predict": 20,
            "draft_model": draft_model_candidate,
            "num_draft": 5
        }
    });
    let draft_resp: OllamaGenerateResponse = client
        .post(&format!("{base_url}/api/generate"))
        .json(&draft_body)
        .send()
        .await?
        .error_for_status()
        .map_err(ApiError::from)?
        .json()
        .await?;

    // 4. Compare: look for draft-specific keys in the response
    // Exact match against known speculative decoding response keys.
    // Substring matching risks false positives on unrelated keys.
    const SPEC_DECODE_KEYS: &[&str] = &[
        "draft_model",
        "draft_tokens_accepted",
        "draft_tokens_generated",
        "draft_tokens_rejected",
        "num_draft",
        "speculative",
        "speculative_decoding",
    ];
    let draft_keys: Vec<String> = draft_resp
        .extra
        .keys()
        .filter(|k| SPEC_DECODE_KEYS.contains(&k.as_str()))
        .cloned()
        .collect();

    if !draft_keys.is_empty() {
        return Ok(ProbeResult::Supported {
            ollama_version: ollama_version.clone(),
            api_params: draft_keys,
            draft_model: draft_model_candidate.to_string(),
        });
    }

    // No draft-specific keys. Check if response contains keys absent from baseline.
    let baseline_key_set: std::collections::HashSet<&String> =
        baseline_resp.extra.keys().collect();
    let draft_key_set: std::collections::HashSet<&String> =
        draft_resp.extra.keys().collect();

    let new_keys: Vec<String> = draft_key_set
        .difference(&baseline_key_set)
        .map(|k| (*k).clone())
        .collect();

    if !new_keys.is_empty() {
        // Draft response has keys that baseline doesn't — partial support
        return Ok(ProbeResult::Supported {
            ollama_version: ollama_version.clone(),
            api_params: new_keys,
            draft_model: draft_model_candidate.to_string(),
        });
    }

    let mut baseline_keys: Vec<String> = baseline_resp.extra.keys().cloned().collect();
    let mut draft_extra_keys: Vec<String> = draft_resp.extra.keys().cloned().collect();
    baseline_keys.sort();
    draft_extra_keys.sort();

    // Identical response shape -> silently ignored
    let reason = format!(
        "ollama {} silently ignores draft_model/num_draft options. \
         PR #8134 was closed (Jan 2025). Issue #5800 remains open. \
         No speculative decoding API parameters in /api/generate docs. \
         Baseline response text: '{}', draft response text: '{}'. \
         Baseline extra keys: {:?}, draft extra keys: {:?}.",
        ollama_version,
        truncate(&baseline_resp.response, 80),
        truncate(&draft_resp.response, 80),
        baseline_keys,
        draft_extra_keys,
    );
    Ok(ProbeResult::NotSupported {
        ollama_version,
        reason,
    })
}

fn truncate(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    // Find the last char boundary at or before max_bytes (UTF-8 safe).
    let boundary = s
        .char_indices()
        .take_while(|(i, _)| *i <= max_bytes)
        .last()
        .map_or(0, |(i, _)| i);
    format!("{}...", &s[..boundary])
}

pub type LlmFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ApiError>> + Send + 'a>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeculativeConfig {
    pub speculative_decoding: bool,
    pub draft_model: Option<String>,
    pub num_draft_tokens: u8,
}

impl Default for SpeculativeConfig {
    fn default() -> Self {
        Self {
            speculative_decoding: false,
            draft_model: None,
            num_draft_tokens: 5,
        }
    }
}

impl SpeculativeConfig {
    pub fn new(
        speculative_decoding: bool,
        draft_model: Option<String>,
        num_draft_tokens: u8,
    ) -> Result<Self, String> {
        if num_draft_tokens == 0 {
            return Err("num_draft_tokens must be >= 1".to_string());
        }
        Ok(Self {
            speculative_decoding,
            draft_model,
            num_draft_tokens,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub speculative: SpeculativeConfig,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.9,
            speculative: SpeculativeConfig::default(),
        }
    }
}

pub struct OllamaConfigBuilder {
    temperature: f32,
    top_p: f32,
    speculative: SpeculativeConfig,
}

impl OllamaConfigBuilder {
    fn new() -> Self {
        let defaults = OllamaConfig::default();
        Self {
            temperature: defaults.temperature,
            top_p: defaults.top_p,
            speculative: defaults.speculative,
        }
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = top_p;
        self
    }

    pub fn speculative(mut self, speculative: SpeculativeConfig) -> Self {
        self.speculative = speculative;
        self
    }

    pub fn build(self) -> OllamaConfig {
        OllamaConfig {
            temperature: self.temperature,
            top_p: self.top_p,
            speculative: self.speculative,
        }
    }
}

impl OllamaConfig {
    #[must_use]
    pub fn builder() -> OllamaConfigBuilder {
        OllamaConfigBuilder::new()
    }
}

pub trait LlmProvider: Send + Sync {
    fn complete<'a>(&'a self, prompt: &'a str, config: &'a OllamaConfig) -> LlmFuture<'a, Vec<String>>;
    fn embed<'a>(&'a self, text: &'a str) -> LlmFuture<'a, Vec<f32>>;
    fn token_count(&self, text: &str) -> Result<usize, ApiError>;
    fn token_count_estimate(&self, text: &str) -> usize;
    /// B-10 (DECISION-B): identifier of the loaded model for tier detection.
    ///
    /// Default impl returns `""` so external implementations keep compiling,
    /// but every internal impl in this crate overrides it. Returning empty
    /// is treated by callers as "unknown" — they emit a single tracing::warn
    /// and fall back to a safe default (`micro_7b_12gb`).
    ///
    /// MUST NOT panic. MUST be cheap (no I/O, no allocation beyond the field).
    fn model_name(&self) -> &str {
        ""
    }
}

#[derive(Debug, Clone)]
pub struct OllamaProvider {
    pub model: String,
    pub base_url: String,
    pub config: OllamaConfig,
}

pub struct OllamaProviderBuilder {
    model: String,
    base_url: String,
    config: OllamaConfig,
}

impl OllamaProviderBuilder {
    fn new() -> Self {
        Self {
            model: String::new(),
            base_url: DEFAULT_OLLAMA_BASE_URL.to_string(),
            config: OllamaConfig::default(),
        }
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn config(mut self, config: OllamaConfig) -> Self {
        self.config = config;
        self
    }

    pub fn build(self) -> OllamaProvider {
        OllamaProvider {
            model: self.model,
            base_url: self.base_url,
            config: self.config,
        }
    }
}

impl OllamaProvider {
    #[must_use]
    pub fn builder() -> OllamaProviderBuilder {
        OllamaProviderBuilder::new()
    }
}

// ---------------------------------------------------------------------------
// LlmProvider impls
//
// token_count returns the same heuristic as token_count_estimate (len/4 + 1)
// because the providers crate has no tokenizer dependency available; adding
// tiktoken-rs or tokenizers would be a new dep (requires human approval).
// Callers needing exact counts should use the sidecar tokenizer endpoint.
// This matches VllmProvider's token_count behavior for cross-provider parity.
// ---------------------------------------------------------------------------

/// Default HTTP timeout for ollama generate / embed requests.
///
/// Cold-start ollama loads can exceed 60s on larger quantized models;
/// 120s matches the probe timeout budget and is safe for gemma4:e2b.
const OLLAMA_REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

/// Shape of POST /api/embeddings (stream is always false).
///
/// Grounded: ollama docs (https://github.com/ollama/ollama/blob/main/docs/api.md)
/// and docs.ollama.com/capabilities/embeddings — request is {"model","prompt"},
/// response is {"embedding": [f32..]}. Verified via WebSearch 2026-04.
#[derive(Debug, Deserialize)]
struct OllamaEmbeddingResponse {
    #[serde(default)]
    embedding: Vec<f32>,
}

/// Internal helper: POST /api/generate with stream=false and return the
/// concatenated response text split into a Vec<String> token stream.
///
/// Ollama (non-streaming) returns a single JSON object with a `response`
/// field; we surface it as a one-element Vec<String> so callers get the
/// same shape as the streaming SSE flow used by other providers.
async fn ollama_complete(
    base_url: &str,
    model: &str,
    prompt: &str,
    config: &OllamaConfig,
) -> Result<Vec<String>, ApiError> {
    let client = reqwest::Client::builder()
        .timeout(OLLAMA_REQUEST_TIMEOUT)
        .build()?;

    let mut options = serde_json::Map::new();
    options.insert(
        "temperature".to_string(),
        serde_json::json!(config.temperature),
    );
    options.insert("top_p".to_string(), serde_json::json!(config.top_p));
    if config.speculative.speculative_decoding {
        if let Some(draft) = &config.speculative.draft_model {
            options.insert("draft_model".to_string(), serde_json::json!(draft));
            options.insert(
                "num_draft".to_string(),
                serde_json::json!(config.speculative.num_draft_tokens),
            );
        }
    }

    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "options": serde_json::Value::Object(options),
    });

    let url = format!("{base_url}/api/generate");
    let response = client.post(&url).json(&body).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<unreadable>".to_string());
        tracing::error!(
            target: "providers::ollama",
            %status,
            %body,
            "ollama /api/generate returned non-success"
        );
        return Err(ApiError::Api {
            status,
            error_type: None,
            message: None,
            body,
            retryable: status.is_server_error(),
            retry_after: None,
        });
    }

    let parsed: OllamaGenerateResponse = response.json().await?;
    Ok(vec![parsed.response])
}

/// Internal helper: POST /api/embeddings and return the embedding vector.
async fn ollama_embed(
    base_url: &str,
    model: &str,
    text: &str,
) -> Result<Vec<f32>, ApiError> {
    let client = reqwest::Client::builder()
        .timeout(OLLAMA_REQUEST_TIMEOUT)
        .build()?;

    let body = serde_json::json!({
        "model": model,
        "prompt": text,
    });

    let url = format!("{base_url}/api/embeddings");
    let response = client.post(&url).json(&body).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<unreadable>".to_string());
        tracing::error!(
            target: "providers::ollama",
            %status,
            %body,
            "ollama /api/embeddings returned non-success"
        );
        return Err(ApiError::Api {
            status,
            error_type: None,
            message: None,
            body,
            retryable: status.is_server_error(),
            retry_after: None,
        });
    }

    let parsed: OllamaEmbeddingResponse = response.json().await?;
    if parsed.embedding.is_empty() {
        tracing::warn!(
            target: "providers::ollama",
            %model,
            "ollama returned empty embedding vector — model may not support embeddings"
        );
        return Err(ApiError::Api {
            status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            error_type: Some("empty_embedding".to_string()),
            message: Some(format!(
                "model {model} returned empty embedding vector"
            )),
            body: String::new(),
            retry_after: None,
            retryable: false,
        });
    }
    Ok(parsed.embedding)
}

/// Rough token estimate: ~4 chars per token (GPT-style heuristic).
///
/// Accurate counting requires a tokenizer — see comment at top of impl block.
#[inline]
fn ollama_token_estimate(text: &str) -> usize {
    text.len() / 4 + 1
}

impl LlmProvider for OllamaProvider {
    fn complete<'a>(
        &'a self,
        prompt: &'a str,
        config: &'a OllamaConfig,
    ) -> LlmFuture<'a, Vec<String>> {
        Box::pin(
            async move { ollama_complete(&self.base_url, &self.model, prompt, config).await },
        )
    }

    fn embed<'a>(&'a self, text: &'a str) -> LlmFuture<'a, Vec<f32>> {
        Box::pin(async move { ollama_embed(&self.base_url, &self.model, text).await })
    }

    fn token_count(&self, text: &str) -> Result<usize, ApiError> {
        Ok(ollama_token_estimate(text))
    }

    fn token_count_estimate(&self, text: &str) -> usize {
        ollama_token_estimate(text)
    }

    /// B-10 (DECISION-B): expose the loaded Ollama model identifier.
    fn model_name(&self) -> &str {
        &self.model
    }
}

#[derive(Debug, Clone)]
pub struct OllamaEmbedder {
    pub model: String,
    pub base_url: String,
    pub config: OllamaConfig,
}

pub struct OllamaEmbedderBuilder {
    model: String,
    base_url: String,
    config: OllamaConfig,
}

impl OllamaEmbedderBuilder {
    fn new() -> Self {
        Self {
            model: String::new(),
            base_url: DEFAULT_OLLAMA_BASE_URL.to_string(),
            config: OllamaConfig::default(),
        }
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn config(mut self, config: OllamaConfig) -> Self {
        self.config = config;
        self
    }

    pub fn build(self) -> OllamaEmbedder {
        OllamaEmbedder {
            model: self.model,
            base_url: self.base_url,
            config: self.config,
        }
    }
}

impl OllamaEmbedder {
    #[must_use]
    pub fn builder() -> OllamaEmbedderBuilder {
        OllamaEmbedderBuilder::new()
    }
}

impl LlmProvider for OllamaEmbedder {
    /// OllamaEmbedder is embedding-only. `complete` is intentionally unsupported
    /// and returns a structured error rather than panicking.
    fn complete<'a>(
        &'a self,
        _prompt: &'a str,
        _config: &'a OllamaConfig,
    ) -> LlmFuture<'a, Vec<String>> {
        Box::pin(async move {
            Err(ApiError::Api {
                status: reqwest::StatusCode::METHOD_NOT_ALLOWED,
                error_type: Some("unsupported_operation".to_string()),
                message: Some(
                    "OllamaEmbedder does not implement complete; use OllamaProvider"
                        .to_string(),
                ),
                body: String::new(),
                retryable: false,
                retry_after: None,
            })
        })
    }

    fn embed<'a>(&'a self, text: &'a str) -> LlmFuture<'a, Vec<f32>> {
        Box::pin(async move { ollama_embed(&self.base_url, &self.model, text).await })
    }

    fn token_count(&self, text: &str) -> Result<usize, ApiError> {
        Ok(ollama_token_estimate(text))
    }

    fn token_count_estimate(&self, text: &str) -> usize {
        ollama_token_estimate(text)
    }

    /// B-10 (DECISION-B): expose the loaded Ollama embedding model identifier.
    fn model_name(&self) -> &str {
        &self.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speculative_config_defaults() {
        let config = SpeculativeConfig::default();
        assert!(!config.speculative_decoding);
        assert!(config.draft_model.is_none());
        assert_eq!(config.num_draft_tokens, 5);
    }

    #[test]
    fn test_speculative_config_serde_roundtrip() {
        let config = SpeculativeConfig {
            speculative_decoding: true,
            draft_model: Some("qwen2:1.5b".to_string()),
            num_draft_tokens: 8,
        };
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: SpeculativeConfig =
            serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.speculative_decoding);
        assert_eq!(
            deserialized.draft_model.as_deref(),
            Some("qwen2:1.5b")
        );
        assert_eq!(deserialized.num_draft_tokens, 8);
    }

    #[test]
    fn test_speculative_config_rejects_zero_draft_tokens() {
        let result = SpeculativeConfig::new(false, None, 0);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("num_draft_tokens must be >= 1")
        );
    }

    // -- Phase 1F.2: ProbeResult tests --

    #[test]
    fn test_probe_result_not_supported_display() {
        let result = ProbeResult::NotSupported {
            ollama_version: "0.20.2".to_string(),
            reason: "silently ignores draft_model".to_string(),
        };
        let display = result.to_string();
        assert!(display.starts_with("NOT_SUPPORTED:"));
        assert!(display.contains("0.20.2"));
        assert!(display.contains("silently ignores"));
    }

    #[test]
    fn test_probe_result_supported_display() {
        let result = ProbeResult::Supported {
            ollama_version: "1.0.0".to_string(),
            api_params: vec!["draft_model".to_string(), "num_draft".to_string()],
            draft_model: "gemma3:2b".to_string(),
        };
        let display = result.to_string();
        assert!(display.starts_with("SUPPORTED:"));
        assert!(display.contains("draft_model, num_draft"));
        assert!(display.contains("gemma3:2b"));
    }

    #[test]
    fn test_probe_result_serde_roundtrip_not_supported() {
        let result = ProbeResult::NotSupported {
            ollama_version: "0.20.2".to_string(),
            reason: "no API support".to_string(),
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: ProbeResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, result);
    }

    #[test]
    fn test_probe_result_serde_roundtrip_supported() {
        let result = ProbeResult::Supported {
            ollama_version: "1.0.0".to_string(),
            api_params: vec!["draft_model".to_string()],
            draft_model: "test:0.5b".to_string(),
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: ProbeResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, result);
    }

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        let long = "a".repeat(100);
        let result = truncate(&long, 10);
        assert_eq!(result.len(), 13); // 10 + "..."
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_truncate_multibyte_utf8() {
        // Each CJK char is 3 bytes. "Hello" = 5 bytes, "世界" = 6 bytes = 11 total.
        let s = "Hello世界!";
        let result = truncate(s, 8); // 8 bytes: "Hello" (5) + "世" (3) = 8 fits
        assert!(result.ends_with("..."));
        // Must not panic and must cut at a char boundary
        assert!(result.starts_with("Hello世"));
    }

    /// Integration test: probes the local ollama instance.
    /// Requires ollama running with gemma4:e2b loaded.
    /// Run with: cargo test --package elai-providers -- --ignored test_probe_local_ollama
    #[tokio::test]
    #[ignore]
    async fn test_probe_local_ollama() {
        let result = probe_speculative_support(
            DEFAULT_OLLAMA_BASE_URL,
            "gemma4:e2b",
            "gemma3:4b",
            std::time::Duration::from_secs(300),
        )
        .await
        .expect("probe should not error");

        println!("{result}");

        // The probe must return a definitive result
        match &result {
            ProbeResult::Supported { ollama_version, api_params, draft_model } => {
                assert!(!ollama_version.is_empty());
                assert!(!api_params.is_empty());
                assert!(!draft_model.is_empty());
            }
            ProbeResult::NotSupported { ollama_version, reason } => {
                assert!(!ollama_version.is_empty());
                assert!(!reason.is_empty());
            }
        }
    }
}
