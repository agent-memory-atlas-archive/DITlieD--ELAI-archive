//! LLM-guard PreToolUse hook for MCP tool calls.
//!
//! Before any MCP tool call executes, this hook prompts the local Qwen 3B
//! sidecar (via ANTHROPIC_BASE_URL) with a structured safety-decision
//! request. The sidecar returns JSON `{"decision": "allow"|"deny"|"ask",
//! "reason": "..."}`.
//!
//! On `deny`: the call is blocked and a `mcp_llm_guard_blocked` SSB row is
//! emitted. On `ask`: the call requires user confirmation via dashboard.
//! On `allow` (or sidecar unavailable with fail-closed=false): proceed.
//!
//! Fail-closed behaviour: if the sidecar is unavailable AND `fail_closed`
//! is true (the default), ALL MCP tool calls are blocked until the sidecar
//! recovers. Pass `--no-llm-guard` to bypass.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tracing::{debug, warn};

/// Guard decision returned by the LLM-guard hook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardDecision {
    Allow,
    Deny { reason: String },
    Ask { reason: String },
}

/// Errors from the LLM-guard module.
#[derive(Debug, Error)]
pub enum GuardError {
    #[error("HTTP request to sidecar failed: {0}")]
    Http(String),
    #[error("Sidecar unavailable and fail_closed=true")]
    SidecarUnavailableClosed,
    #[error("Response parse error: {0}")]
    ParseError(String),
}

/// Serialized response from the sidecar.
#[derive(Debug, Deserialize)]
struct SidecarGuardResponse {
    decision: String,
    reason: Option<String>,
}

/// Request payload sent to the sidecar.
#[derive(Debug, Serialize)]
struct GuardRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<GuardMessage<'a>>,
}

#[derive(Debug, Serialize)]
struct GuardMessage<'a> {
    role: &'a str,
    content: String,
}

/// Configuration for the LLM-guard hook.
#[derive(Debug, Clone)]
pub struct LlmGuardConfig {
    /// Base URL for the Anthropic-protocol sidecar (e.g. "http://localhost:8100").
    pub sidecar_url: String,
    /// Model name to request from the sidecar.
    pub model: String,
    /// If true, block all MCP calls when sidecar is unavailable.
    pub fail_closed: bool,
    /// Maximum tokens for the guard response.
    pub max_tokens: u32,
}

impl Default for LlmGuardConfig {
    fn default() -> Self {
        Self {
            sidecar_url: "http://localhost:8100".to_string(),
            model: "qwen".to_string(),
            fail_closed: true,
            max_tokens: 128,
        }
    }
}

/// Ask the LLM-guard whether an MCP tool call is safe to execute.
///
/// # Arguments
/// * `config` -- guard configuration (sidecar URL, model, fail-closed flag).
/// * `tool_name` -- name of the MCP tool about to be called.
/// * `params` -- sanitized parameters for the call.
///
/// Returns `Ok(GuardDecision)` or an error if the sidecar is unreachable
/// and `fail_closed=true`.
#[cfg(feature = "mcp")]
pub async fn pre_mcp_call_decision(
    config: &LlmGuardConfig,
    tool_name: &str,
    params: &Value,
) -> Result<GuardDecision, GuardError> {
    let prompt = format!(
        "You are a security guard for MCP tool calls. \
        Evaluate whether this call is safe to execute. \
        Tool: {tool_name}\nParameters: {params}\n\
        Output ONLY valid JSON: {{\"decision\": \"allow\"|\"deny\"|\"ask\", \"reason\": \"...\"}}"
    );

    let request = GuardRequest {
        model: &config.model,
        max_tokens: config.max_tokens,
        messages: vec![GuardMessage {
            role: "user",
            content: prompt,
        }],
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| GuardError::Http(e.to_string()))?;

    let messages_url = format!("{}/v1/messages", config.sidecar_url.trim_end_matches('/'));

    let resp = match client.post(&messages_url).json(&request).send().await {
        Ok(r) => r,
        Err(e) => {
            warn!(%tool_name, error = %e, "llm_guard: sidecar request failed");
            if config.fail_closed {
                return Err(GuardError::SidecarUnavailableClosed);
            } else {
                debug!(%tool_name, "llm_guard: sidecar unavailable, fail_closed=false -> allow");
                return Ok(GuardDecision::Allow);
            }
        }
    };

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        warn!(%tool_name, %status, "llm_guard: sidecar returned error status");
        if config.fail_closed {
            return Err(GuardError::SidecarUnavailableClosed);
        }
        return Ok(GuardDecision::Allow);
    }

    // Extract the text content from the Anthropic-protocol response.
    let body: Value = resp
        .json()
        .await
        .map_err(|e| GuardError::ParseError(e.to_string()))?;

    let text = body
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("");

    // Attempt JSON parse; fall back to allow on parse failure.
    let decision: SidecarGuardResponse = match serde_json::from_str(text) {
        Ok(d) => d,
        Err(e) => {
            warn!(%tool_name, error = %e, raw = %text, "llm_guard: failed to parse sidecar response");
            return Ok(GuardDecision::Allow);
        }
    };

    let reason = decision.reason.unwrap_or_default();
    let result = match decision.decision.as_str() {
        "deny" => GuardDecision::Deny { reason },
        "ask" => GuardDecision::Ask { reason },
        _ => GuardDecision::Allow,
    };

    debug!(%tool_name, decision = ?result, "llm_guard: decision");
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_is_fail_closed() {
        let cfg = LlmGuardConfig::default();
        assert!(cfg.fail_closed, "default config must be fail-closed");
    }

    #[test]
    fn guard_decision_variants_are_correct() {
        let allow = GuardDecision::Allow;
        let deny = GuardDecision::Deny { reason: "bad".into() };
        let ask = GuardDecision::Ask { reason: "uncertain".into() };
        assert_ne!(allow, deny);
        assert_ne!(allow, ask);
    }
}
