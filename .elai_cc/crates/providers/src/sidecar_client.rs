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
    /// Batch 4A (3J): HyDE enabled for this dispatch. Forwarded to the sidecar
    /// so the engine can decide whether to call `HydeStage::hypothesize`
    /// instead of the direct search path. Default: `false` (skipped unless
    /// the dispatch context enables it on DEEP-tier queries).
    #[serde(default, skip_serializing_if = "is_default_bool")]
    pub hyde_enabled: bool,
    /// Batch 4A (3J): ActAdd steering add-on. Distinct from the legacy
    /// `steering_enabled` flag which drives the global ActAdd vector cache —
    /// this per-dispatch flag lets the orchestrator opt in to a per-role
    /// ActAdd payload without flipping the global switch.
    #[serde(default, skip_serializing_if = "is_default_bool")]
    pub actadd_enabled: bool,
    /// Batch 4A (3J): target layer for the per-dispatch ActAdd payload. Only
    /// serialized when `actadd_enabled` is true — default stays implicit.
    #[serde(default, skip_serializing_if = "is_default_u32")]
    pub actadd_layer: u32,
}

fn is_default_bool(b: &bool) -> bool {
    !*b
}

fn is_default_u32(n: &u32) -> bool {
    *n == 0
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
            hyde_enabled: false,
            actadd_enabled: false,
            actadd_layer: 0,
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

/// GG intrinsic verification signals (A-17, A-25).
///
/// Extracted from token logprobs and entropy during generation at zero
/// additional inference cost. Used for AB-MCTS branch scoring (Layer 4).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GGSignals {
    /// Mean log-probability of generated tokens (higher = more confident).
    pub mean_logprob: Option<f64>,
    /// Minimum (worst) log-probability among generated tokens.
    pub min_logprob: Option<f64>,
    /// Mean token-level entropy across the generation.
    pub entropy_mean: Option<f64>,
    /// Peak (maximum) token-level entropy during generation.
    pub entropy_max: Option<f64>,
    /// Generation novelty (A-25): 1.0 - cosine_sim(current, previous) for same prompt.
    /// 0.0 = identical to previous attempt, 1.0 = completely different.
    /// None on first generation for a given prompt.
    #[serde(default)]
    pub generation_novelty: Option<f64>,
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
    /// GG intrinsic verification signals (A-17).
    #[serde(default)]
    mean_logprob: Option<f64>,
    #[serde(default)]
    min_logprob: Option<f64>,
    #[serde(default)]
    entropy_mean: Option<f64>,
    #[serde(default)]
    entropy_max: Option<f64>,
    /// Generation novelty signal (A-25).
    #[serde(default)]
    generation_novelty: Option<f64>,
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
    /// Last GG intrinsic verification signals from the most recent completion.
    last_gg_signals: Mutex<GGSignals>,
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
            last_gg_signals: Mutex::new(GGSignals::default()),
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

    /// Build the JSON body for a `/v1/completions` request.
    ///
    /// patch-1.33: forwards `dola_enabled`, `dola_alpha`, `turboquant_bits`,
    /// `steering_enabled`, `steering_coefficient`, `steering_layer`, and
    /// `etc_threshold` from `SidecarConfig` into the request body so the
    /// sidecar can report (and future patches can honour) per-request layer
    /// overrides. `etc_threshold` serializes `f64::INFINITY` as `null` to
    /// preserve the disabled-until-calibration contract.
    pub(crate) fn build_completion_body(config: &SidecarConfig, prompt: &str) -> serde_json::Value {
        let etc_threshold_json: serde_json::Value = if config.etc_threshold.is_infinite() {
            serde_json::Value::Null
        } else {
            serde_json::json!(config.etc_threshold)
        };
        let mut body = serde_json::json!({
            "prompt": prompt,
            "max_tokens": config.max_tokens,
            "temperature": config.temperature,
            "dola_enabled": config.dola_enabled,
            "dola_alpha": config.dola_alpha,
            "turboquant_bits": config.turboquant_bits,
            "steering_enabled": config.steering_enabled,
            "steering_coefficient": config.steering_coefficient,
            "steering_layer": config.steering_layer,
            "etc_threshold": etc_threshold_json,
        });
        // Batch 4A (3J): include optional per-dispatch knobs only when they
        // deviate from the default so the wire payload stays compact. Matches
        // the `skip_serializing_if` gating on `SidecarConfig` for TOML I/O.
        if let Some(obj) = body.as_object_mut() {
            if config.hyde_enabled {
                obj.insert("hyde_enabled".to_owned(), serde_json::Value::Bool(true));
            }
            if config.actadd_enabled {
                obj.insert("actadd_enabled".to_owned(), serde_json::Value::Bool(true));
                obj.insert(
                    "actadd_layer".to_owned(),
                    serde_json::Value::Number(config.actadd_layer.into()),
                );
            }
        }
        body
    }

    /// Send a completion request to /v1/completions.
    ///
    /// Returns (completions, optional ETC search trigger, GG signals).
    async fn complete_impl(
        &self,
        prompt: &str,
    ) -> Result<(Vec<String>, Option<SearchTriggerPayload>, GGSignals), ApiError> {
        let url = format!("{}/v1/completions", self.config.server_url);
        let body = Self::build_completion_body(&self.config, prompt);

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
        let gg = GGSignals {
            mean_logprob: resp.mean_logprob,
            min_logprob: resp.min_logprob,
            entropy_mean: resp.entropy_mean,
            entropy_max: resp.entropy_max,
            generation_novelty: resp.generation_novelty,
        };
        Ok((tokens, resp.search_trigger, gg))
    }

    /// Access the last ETC search trigger from the most recent completion.
    /// Callers should check this after `complete()` to decide on search injection.
    pub fn last_search_trigger(&self) -> Option<SearchTriggerPayload> {
        self.last_search_trigger
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
    }

    /// Access GG intrinsic verification signals from the most recent completion.
    /// Used for AB-MCTS branch scoring (Layer 4).
    pub fn last_gg_signals(&self) -> GGSignals {
        self.last_gg_signals
            .lock()
            .ok()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    /// Borrow the current sidecar config.
    pub fn config(&self) -> &SidecarConfig {
        &self.config
    }

    /// Batch 4A (3J): check whether a per-role steering vector exists on disk
    /// under `<project_root>/.elai_cc/steering/<project_id>/<role>.bin`.
    /// Returns `true` when the file exists and is non-empty. Used by the
    /// orchestrator dispatch path to decide whether to attach an ActAdd
    /// payload without blocking when the cache is cold.
    ///
    /// This is a lookup-only helper — computing a fresh vector via
    /// `compute_vector_via_sidecar` is deferred per the A-8 note in the
    /// batch brief.
    pub fn steering_vector_cached(
        project_root: &std::path::Path,
        project_id: &str,
        role: &str,
    ) -> bool {
        let path = steering_cache_path(project_root, project_id, role);
        std::fs::metadata(&path).map(|m| m.len() > 0).unwrap_or(false)
    }
}

/// Batch 4A (3J): canonical filesystem location of a per-role steering
/// vector. Separate free function so tests and the dashboard helper can
/// both compute the same path without cloning provider state.
pub fn steering_cache_path(
    project_root: &std::path::Path,
    project_id: &str,
    role: &str,
) -> std::path::PathBuf {
    project_root
        .join(".elai_cc")
        .join("steering")
        .join(project_id)
        .join(format!("{role}.bin"))
}

/// Batch 4A (3J): snapshot of the optimizations currently active for a
/// dispatch. Surfaced by the dashboard StatusBar so users see the model +
/// TQ bits + DoLa on/off + steering role at a glance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OptimizationSummary {
    pub model: String,
    pub turboquant_bits: u8,
    pub dola_enabled: bool,
    pub steering_enabled: bool,
    pub actadd_enabled: bool,
    pub hyde_enabled: bool,
    /// Optional role label when a per-role steering vector is cached
    /// (e.g. `"ada"` for an ActAdd ada-boost role). `None` means the
    /// steering vector cache miss.
    pub steering_role: Option<String>,
}

impl OptimizationSummary {
    /// Build a summary from a SidecarConfig + an optional active role.
    pub fn from_config(config: &SidecarConfig, steering_role: Option<String>) -> Self {
        Self {
            model: config.model.clone(),
            turboquant_bits: config.turboquant_bits,
            dola_enabled: config.dola_enabled,
            steering_enabled: config.steering_enabled,
            actadd_enabled: config.actadd_enabled,
            hyde_enabled: config.hyde_enabled,
            steering_role,
        }
    }

    /// Compact one-line formatter used by the dashboard StatusBar. Example:
    /// `model: qwen2.5-coder-7b · [TQ:4, DoLa:off, Steer:ada]`. Matches the
    /// format the batch-4a brief lists for the StatusBar row.
    pub fn to_status_line(&self) -> String {
        let dola = if self.dola_enabled { "on" } else { "off" };
        let steer = match &self.steering_role {
            Some(role) if self.steering_enabled => role.clone(),
            Some(_) => "cached".to_owned(),
            None if self.steering_enabled => "on".to_owned(),
            None => "off".to_owned(),
        };
        let model_label = if self.model.is_empty() {
            "sidecar".to_owned()
        } else {
            self.model.clone()
        };
        format!(
            "model: {} · [TQ:{}, DoLa:{}, Steer:{}]",
            model_label, self.turboquant_bits, dola, steer,
        )
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
                Ok((completions, trigger, gg)) => {
                    let triggered = trigger.is_some();
                    if let Ok(mut guard) = self.last_search_trigger.lock() {
                        *guard = trigger;
                    }
                    if let Ok(mut guard) = self.last_gg_signals.lock() {
                        *guard = gg;
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
            hyde_enabled: false,
            actadd_enabled: false,
            actadd_layer: 0,
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
    fn completion_body_includes_all_layer_config_fields() {
        // patch-1.33 contract: the request body must carry the seven per-
        // request layer/TQ/ETC knobs so the sidecar engine can honour them
        // (or at least log them) instead of falling back to startup defaults.
        let config = SidecarConfig {
            server_url: "http://localhost:8100".into(),
            model: "qwen-coder-7b".into(),
            dola_enabled: true,
            dola_alpha: 0.05,
            turboquant_bits: 4,
            steering_enabled: true,
            steering_coefficient: 7.5,
            steering_layer: "12".into(),
            temperature: 0.3,
            max_tokens: 64,
            etc_threshold: 1.25,
            hyde_enabled: false,
            actadd_enabled: false,
            actadd_layer: 0,
        };
        let body = SidecarProvider::build_completion_body(&config, "hello");
        assert_eq!(body["prompt"], "hello");
        assert_eq!(body["max_tokens"], 64);
        assert!((body["temperature"].as_f64().unwrap() - 0.3).abs() < 1e-6);
        assert_eq!(body["dola_enabled"], true);
        assert!((body["dola_alpha"].as_f64().unwrap() - 0.05).abs() < 1e-9);
        assert_eq!(body["turboquant_bits"], 4);
        assert_eq!(body["steering_enabled"], true);
        assert!((body["steering_coefficient"].as_f64().unwrap() - 7.5).abs() < 1e-9);
        assert_eq!(body["steering_layer"], "12");
        assert!((body["etc_threshold"].as_f64().unwrap() - 1.25).abs() < 1e-9);
    }

    #[test]
    fn completion_body_etc_threshold_infinity_serializes_as_null() {
        let config = SidecarConfig::default();
        let body = SidecarProvider::build_completion_body(&config, "x");
        assert!(body["etc_threshold"].is_null());
    }

    #[test]
    fn search_trigger_payload_deserialization() {
        let json = r#"{"partial_text":"import ","detected_context":"import","trigger_token_idx":42}"#;
        let payload: SearchTriggerPayload = serde_json::from_str(json).expect("deserialize");
        assert_eq!(payload.partial_text, "import ");
        assert_eq!(payload.detected_context, "import");
        assert_eq!(payload.trigger_token_idx, 42);
    }

    #[test]
    fn gg_signals_default() {
        let gg = GGSignals::default();
        assert!(gg.mean_logprob.is_none());
        assert!(gg.min_logprob.is_none());
        assert!(gg.entropy_mean.is_none());
        assert!(gg.entropy_max.is_none());
        assert!(gg.generation_novelty.is_none());
    }

    #[test]
    fn completion_response_with_gg_signals() {
        let json = r#"{
            "choices": [{"text": "fn main() {}", "finish_reason": "stop"}],
            "search_trigger": null,
            "mean_logprob": -1.23,
            "min_logprob": -4.56,
            "entropy_mean": 2.1,
            "entropy_max": 5.8,
            "generation_novelty": 0.73
        }"#;
        let resp: CompletionResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.choices.len(), 1);
        assert!((resp.mean_logprob.unwrap() - (-1.23)).abs() < f64::EPSILON);
        assert!((resp.min_logprob.unwrap() - (-4.56)).abs() < f64::EPSILON);
        assert!((resp.entropy_mean.unwrap() - 2.1).abs() < f64::EPSILON);
        assert!((resp.entropy_max.unwrap() - 5.8).abs() < f64::EPSILON);
        assert!((resp.generation_novelty.unwrap() - 0.73).abs() < f64::EPSILON);
    }

    #[test]
    fn completion_response_without_gg_signals() {
        let json = r#"{
            "choices": [{"text": "hello", "finish_reason": "length"}],
            "search_trigger": null
        }"#;
        let resp: CompletionResponse = serde_json::from_str(json).expect("deserialize");
        assert!(resp.mean_logprob.is_none());
        assert!(resp.min_logprob.is_none());
        assert!(resp.entropy_mean.is_none());
        assert!(resp.entropy_max.is_none());
        assert!(resp.generation_novelty.is_none());
    }

    #[test]
    fn gg_signals_with_novelty_roundtrip() {
        let gg = GGSignals {
            mean_logprob: Some(-1.5),
            min_logprob: Some(-3.0),
            entropy_mean: Some(1.8),
            entropy_max: Some(4.2),
            generation_novelty: Some(0.65),
        };
        let json = serde_json::to_string(&gg).expect("serialize");
        let back: GGSignals = serde_json::from_str(&json).expect("deserialize");
        assert!((back.generation_novelty.unwrap() - 0.65).abs() < f64::EPSILON);
    }
}
