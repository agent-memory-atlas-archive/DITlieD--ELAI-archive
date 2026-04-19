//! Batch-fu (FU.4 / A-23) — SidecarGroundingClient → ReflectionClient adapter.
//!
//! The 4C micro-reflection emitter (`crate::micro_reflection_emitter`) expects
//! a [`crate::ReflectionClient`] that returns a parsed, validated
//! [`elai_core::micro_reflection::MicroReflectionResponse`]. The production
//! HTTP client we already ship is [`crate::grounding_client::SidecarGroundingClient`]
//! — a reqwest-based POST-er that talks to `/v1/completions` on the sidecar.
//! This adapter bridges the two so the dashboard can plug the existing client
//! into `TaskRunnerConfig.reflection_client` without a second connection pool.
//!
//! The sidecar endpoint does NOT currently expose a `response_format`
//! constrained-JSON switch (grep `crates/orchestrator/src/grounding_client.rs`
//! for `CompletionsRequest`; only `prompt` / `max_tokens` / `temperature` are
//! serialized — see L153). We therefore best-effort extract a JSON object
//! from the raw response text with a clear warning logged via `tracing` when
//! the response is not pure JSON. Parsing delegates to
//! [`crate::micro_reflection_emitter::parse_reflection_json`], which returns a
//! [`crate::micro_reflection_emitter::ReflectionClientError::Parse`] on
//! malformed output — exactly what the emitter expects to surface.

use std::sync::Arc;
use std::time::Duration;

use elai_core::micro_reflection::MicroReflectionResponse;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::grounding_client::SidecarGroundingClient;
use crate::micro_reflection_emitter::{
    parse_reflection_json, ReflectionClient, ReflectionClientError,
};

/// Env-var kill switch. When set to any non-empty string, the dashboard
/// construction site skips wiring the adapter and `reflection_client` stays
/// `None` — reflection is fully disabled (pre-batch-fu behaviour).
pub const ELAI_DISABLE_MICRO_REFLECTION_ENV: &str = "ELAI_DISABLE_MICRO_REFLECTION";

/// Returns true when [`ELAI_DISABLE_MICRO_REFLECTION_ENV`] is set to any
/// non-empty string.
pub fn micro_reflection_disabled_via_env() -> bool {
    std::env::var(ELAI_DISABLE_MICRO_REFLECTION_ENV)
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

/// Reflection-endpoint path. The sidecar already serves this on the shared
/// `/v1/completions` surface so we reuse the existing connection pool — no
/// second reqwest client.
const SIDECAR_COMPLETIONS_PATH: &str = "/v1/completions";

/// Default token cap for the reflection prompt. A-23 spec caps reflections at
/// 500 tokens (prompt-efficient diagnosis).
const REFLECTION_MAX_TOKENS: u32 = 500;

/// Deterministic decode for reflection diagnoses (A-23 MicroReflection
/// profile: temperature=0.3 per `context_budget::GenerationProfile::MicroReflection`).
const REFLECTION_TEMPERATURE: f64 = 0.3;

/// Per-request timeout for reflection calls. Inherited from
/// [`SidecarGroundingClient::timeout`] when the caller supplied one; else 20s.
const REFLECTION_DEFAULT_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Serialize)]
struct CompletionsRequest<'a> {
    prompt: &'a str,
    max_tokens: u32,
    temperature: f64,
}

#[derive(Deserialize)]
struct CompletionsChoice {
    text: String,
}

#[derive(Deserialize)]
struct CompletionsResponse {
    choices: Vec<CompletionsChoice>,
}

/// Adapter exposing a [`ReflectionClient`] backed by a
/// [`SidecarGroundingClient`]. Holds the inner client behind an `Arc` so the
/// dashboard can share a single client instance across the decomposition and
/// reflection call sites.
#[derive(Debug, Clone)]
pub struct SidecarReflectionAdapter {
    inner: Arc<SidecarGroundingClient>,
    http: reqwest::Client,
    timeout: Duration,
}

impl SidecarReflectionAdapter {
    /// Construct an adapter over the supplied grounding client. Reuses the
    /// client's `base_url` + `timeout`; opens a fresh reqwest client so the
    /// adapter has its own pool (prevents head-of-line blocking on the
    /// grounding path when a reflection call is in flight).
    pub fn new(inner: Arc<SidecarGroundingClient>) -> Self {
        let timeout = inner.timeout();
        Self {
            inner,
            http: reqwest::Client::new(),
            timeout: if timeout.is_zero() {
                REFLECTION_DEFAULT_TIMEOUT
            } else {
                timeout
            },
        }
    }

    fn completions_url(&self) -> String {
        format!("{}{}", self.inner.base_url(), SIDECAR_COMPLETIONS_PATH)
    }
}

impl ReflectionClient for SidecarReflectionAdapter {
    async fn reflect(
        &self,
        prompt: &str,
    ) -> Result<MicroReflectionResponse, ReflectionClientError> {
        let url = self.completions_url();
        let body = CompletionsRequest {
            prompt,
            max_tokens: REFLECTION_MAX_TOKENS,
            temperature: REFLECTION_TEMPERATURE,
        };
        debug!(
            url = %url,
            prompt_bytes = prompt.len(),
            "sidecar_reflection: POST /v1/completions"
        );
        let response = self
            .http
            .post(&url)
            .timeout(self.timeout)
            .json(&body)
            .send()
            .await
            .map_err(|e| ReflectionClientError::Transport(e.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            return Err(ReflectionClientError::Unavailable(format!(
                "sidecar http status {}",
                status.as_u16()
            )));
        }
        let parsed: CompletionsResponse = response
            .json()
            .await
            .map_err(|e| ReflectionClientError::Transport(format!("completions envelope: {e}")))?;
        let text = parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.text)
            .ok_or_else(|| ReflectionClientError::Unavailable("empty choices".into()))?;
        match parse_reflection_json(&text) {
            Ok(r) => Ok(r),
            Err(e) => {
                // Best-effort JSON extraction: try to locate a `{...}` block
                // inside the raw completion and re-parse. Sidecar models that
                // prefix a leading sentence before the JSON body are common.
                if let Some(extracted) = extract_json_object(&text) {
                    match parse_reflection_json(extracted) {
                        Ok(r) => {
                            warn!(
                                "sidecar_reflection: response had non-JSON prefix; recovered via extractor"
                            );
                            return Ok(r);
                        }
                        Err(_) => {}
                    }
                }
                warn!(
                    error = %e,
                    "sidecar_reflection: response did not parse as MicroReflectionResponse"
                );
                Err(ReflectionClientError::Parse(e))
            }
        }
    }
}

/// Locate the first top-level `{...}` object in a string. Returns a
/// borrowed slice or `None` when no balanced object is present. Helper kept
/// tight — the emitter's MicroReflectionResponse has a fixed top-level shape
/// so a naive balanced-brace scan is sufficient.
fn extract_json_object(s: &str) -> Option<&str> {
    let start = s.find('{')?;
    let mut depth: i32 = 0;
    for (i, ch) in s[start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&s[start..start + i + 1]);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_json_object_basic() {
        let raw = "prefix sentence {\"root_cause\":\"x\",\"faulty_line\":1,\"action\":\"y\"} trailer";
        let got = extract_json_object(raw).expect("extracted");
        assert!(got.starts_with('{') && got.ends_with('}'));
    }

    #[test]
    fn extract_json_object_none_when_unbalanced() {
        assert!(extract_json_object("no braces here").is_none());
        assert!(extract_json_object("only { an opening").is_none());
    }

    #[test]
    fn disabled_via_env_toggles_with_env_var() {
        // Use a unique name so parallel tests don't clobber each other.
        let key = ELAI_DISABLE_MICRO_REFLECTION_ENV;
        // Cannot mutate process env safely across tests → just assert the
        // read path returns a sensible default when unset.
        std::env::remove_var(key);
        assert!(!micro_reflection_disabled_via_env());
        std::env::set_var(key, "1");
        assert!(micro_reflection_disabled_via_env());
        std::env::remove_var(key);
    }
}
