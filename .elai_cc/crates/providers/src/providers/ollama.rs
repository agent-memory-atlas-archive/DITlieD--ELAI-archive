use std::future::Future;
use std::pin::Pin;

use serde::{Deserialize, Serialize};

use crate::error::ApiError;

pub const DEFAULT_OLLAMA_BASE_URL: &str = "http://localhost:11434";

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

impl LlmProvider for OllamaProvider {
    fn complete<'a>(&'a self, _prompt: &'a str, _config: &'a OllamaConfig) -> LlmFuture<'a, Vec<String>> {
        todo!()
    }

    fn embed<'a>(&'a self, _text: &'a str) -> LlmFuture<'a, Vec<f32>> {
        todo!()
    }

    fn token_count(&self, _text: &str) -> Result<usize, ApiError> {
        todo!()
    }

    fn token_count_estimate(&self, _text: &str) -> usize {
        todo!()
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
    fn complete<'a>(&'a self, _prompt: &'a str, _config: &'a OllamaConfig) -> LlmFuture<'a, Vec<String>> {
        todo!()
    }

    fn embed<'a>(&'a self, _text: &'a str) -> LlmFuture<'a, Vec<f32>> {
        todo!()
    }

    fn token_count(&self, _text: &str) -> Result<usize, ApiError> {
        todo!()
    }

    fn token_count_estimate(&self, _text: &str) -> usize {
        todo!()
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
}
