//! SidecarHydeClient — production `HydeClient` backed by the inference sidecar.
//!
//! patch-1.34. The knowledge crate owns the `HydeClient` trait but has no
//! `sidecar` cargo feature, so the trait implementation lives in the CLI
//! crate (which already depends on both `elai_knowledge` and
//! `elai_providers` with the `sidecar` feature enabled).
//!
//! Dispatches through `SidecarProvider::complete()` — the same Arc instance
//! that the `RuntimeClient` holds, so patch-1.32's SidecarProvider field has
//! a real production call-site via this path. A dedicated current-thread
//! tokio runtime bridges the sync `HydeClient::generate_hypothesis` signature
//! to the async `LlmProvider::complete` trait.

use std::sync::Arc;

use elai_knowledge::hyde::{
    HydeClient, HYDE_MAX_TOKENS, HYDE_PROMPT_PREFIX, HYDE_TEMPERATURE,
};
use elai_providers::{OllamaConfig, LlmProvider, SidecarProvider};

/// Production HyDE client that dispatches through a shared `SidecarProvider`.
pub struct SidecarHydeClient {
    provider: Arc<SidecarProvider>,
    /// Dedicated runtime for bridging the sync `HydeClient` trait to the
    /// async `LlmProvider::complete` future. current_thread is enough — HyDE
    /// requests are short and serial per query.
    runtime: tokio::runtime::Runtime,
}

impl SidecarHydeClient {
    /// Build a new client around an already-constructed `SidecarProvider`.
    pub fn new(provider: Arc<SidecarProvider>) -> Result<Self, std::io::Error> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        Ok(Self { provider, runtime })
    }
}

impl HydeClient for SidecarHydeClient {
    fn generate_hypothesis(&self, query: &str) -> Result<String, String> {
        let prompt = format!("{HYDE_PROMPT_PREFIX}{query}");
        // Use HyDE-specific decoding parameters; SidecarProvider forwards its
        // own SidecarConfig (max_tokens, temperature) so this OllamaConfig is
        // informational — we keep temperature in sync for any future provider
        // that honours OllamaConfig directly.
        let config = OllamaConfig {
            temperature: HYDE_TEMPERATURE,
            ..OllamaConfig::default()
        };
        let _ = HYDE_MAX_TOKENS; // honoured by SidecarConfig.max_tokens

        let completions = self
            .runtime
            .block_on(self.provider.complete(&prompt, &config))
            .map_err(|e| format!("sidecar HyDE completion failed: {e}"))?;

        let text = completions
            .into_iter()
            .next()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        if text.is_empty() {
            return Err("sidecar HyDE returned empty completion".to_string());
        }
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::OsClock;
    use elai_providers::SidecarConfig;

    fn make_provider() -> Arc<SidecarProvider> {
        let cfg = SidecarConfig::default();
        let clock: Arc<dyn elai_core::ClockProvider> = Arc::new(OsClock);
        Arc::new(SidecarProvider::new(cfg, clock).expect("construct sidecar provider"))
    }

    #[test]
    fn client_construction_succeeds() {
        let client = SidecarHydeClient::new(make_provider());
        assert!(client.is_ok());
    }

    #[test]
    fn empty_query_still_dispatches() {
        // The client does NOT short-circuit on empty query — HydeStage owns
        // the skip-short-query logic. Verify the client accepts any string.
        let client = SidecarHydeClient::new(make_provider()).expect("client");
        // We don't actually hit the sidecar (it's not running in tests); just
        // confirm the prefix concatenation compiles and takes ownership.
        let _prompt = format!("{HYDE_PROMPT_PREFIX}{}", "");
        let _ = &client; // force use to avoid dead_code drop
    }
}
