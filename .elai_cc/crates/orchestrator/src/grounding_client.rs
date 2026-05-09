// Session 4D.10: sidecar-backed LLM grounding for recursive research
// decomposition. Replaces the prior synthetic-only path in research_tree.rs
// with a typed, async GroundingClient trait + a production SidecarGroundingClient
// speaking the sidecar's OpenAI-compatible `/v1/completions` endpoint.
//
// Native async fn in traits (no async_trait crate) + generic static dispatch
// per the workspace anti-stale-training rule.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

use crate::research_tree::{
    Alternative, ConvergenceStatus, EvidenceLevel, ResearchReport, ResearchTreeSnapshot,
    SubQuestionResult,
};

// ---------------------------------------------------------------------------
// Constants — named per codeRules (no magic numbers).
// ---------------------------------------------------------------------------

pub const SIDECAR_DEFAULT_HOST: &str = "http://localhost:8100";
pub const SIDECAR_COMPLETIONS_PATH: &str = "/v1/completions";
pub const GROUNDING_TIMEOUT_SECS: u64 = 30;
pub const GROUNDING_MAX_TOKENS: u32 = 2048;
pub const GROUNDING_TEMPERATURE: f64 = 0.0;

const TEMPLATE_L0_IDEA: &str = include_str!("research_prompts/l0_idea.txt");
const TEMPLATE_L1_CONCEPT: &str = include_str!("research_prompts/l1_concept.txt");
const TEMPLATE_L2_PHASE: &str = include_str!("research_prompts/l2_phase.txt");
const TEMPLATE_L3_STEP: &str = include_str!("research_prompts/l3_step.txt");

// ---------------------------------------------------------------------------
// ResearchLevel — which prompt template applies at each recursion depth.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResearchLevel {
    Idea,
    Concept,
    Phase,
    Step,
}

impl ResearchLevel {
    pub fn template_name(self) -> &'static str {
        match self {
            ResearchLevel::Idea => "l0_idea",
            ResearchLevel::Concept => "l1_concept",
            ResearchLevel::Phase => "l2_phase",
            ResearchLevel::Step => "l3_step",
        }
    }

    pub fn template_body(self) -> &'static str {
        match self {
            ResearchLevel::Idea => TEMPLATE_L0_IDEA,
            ResearchLevel::Concept => TEMPLATE_L1_CONCEPT,
            ResearchLevel::Phase => TEMPLATE_L2_PHASE,
            ResearchLevel::Step => TEMPLATE_L3_STEP,
        }
    }

    fn render(self, prompt: &str) -> String {
        self.template_body().replace("{prompt}", prompt)
    }
}

// ---------------------------------------------------------------------------
// GroundingError — typed, Send + Sync + 'static via thiserror.
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum GroundingError {
    #[error("sidecar unavailable at {url}")]
    SidecarUnavailable { url: String },

    #[error("sidecar timeout after {after:?} at {url}")]
    Timeout { url: String, after: Duration },

    #[error("sidecar returned HTTP {status} from {url}")]
    HttpStatus { status: u16, url: String },

    #[error("invalid JSON in sidecar response: {reason}")]
    InvalidJson { reason: String },

    #[error("empty sidecar response")]
    EmptyResponse,

    #[error("sidecar request failed: {reason}")]
    RequestFailed { reason: String },
}

// ---------------------------------------------------------------------------
// GroundingClient — trait. Native async fn (no async_trait). Object safety is
// intentionally not required: callers bind a concrete or generic type.
// ---------------------------------------------------------------------------

pub trait GroundingClient: Send + Sync {
    fn ground(
        &self,
        prompt: &str,
        level: ResearchLevel,
    ) -> impl std::future::Future<Output = Result<ResearchReport, GroundingError>> + Send;

    /// Call the completions endpoint with a raw prompt string (no template wrapping).
    /// Returns the raw text from the first choice. Used by Phase F synthesis which
    /// builds its own detailed instruction prompt.
    fn synthesize_text(
        &self,
        prompt: &str,
    ) -> impl std::future::Future<Output = Result<String, GroundingError>> + Send;
}

// ---------------------------------------------------------------------------
// SidecarGroundingClient — production path. Uses reqwest + tokio.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SidecarGroundingClient {
    base_url: String,
    timeout: Duration,
    http: reqwest::Client,
}

impl SidecarGroundingClient {
    pub fn new(base_url: String, timeout: Duration) -> Self {
        // `Client::new()` is infallible; per-request timeouts are applied via
        // `.timeout()` on the RequestBuilder inside `ground()`.
        Self {
            base_url,
            timeout,
            http: reqwest::Client::new(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    fn completions_url(&self) -> String {
        format!("{}{}", self.base_url, SIDECAR_COMPLETIONS_PATH)
    }
}

impl Default for SidecarGroundingClient {
    fn default() -> Self {
        Self::new(
            SIDECAR_DEFAULT_HOST.to_string(),
            Duration::from_secs(GROUNDING_TIMEOUT_SECS),
        )
    }
}

// Track C-B: opt-in JSON-schema-constrained decoding via lm-format-enforcer.
// `response_format` mirrors the OpenAI / Anthropic-compatible request shape
//   { "type": "json_schema",
//     "json_schema": { "name": "...", "schema": { ... JSON Schema ... } } }
// The sidecar (server.py handle_completions) extracts this dict and threads it
// through InferenceEngine.generate -> model.generate(prefix_allowed_tokens_fn=).
//
// Backward compat: `skip_serializing_if = "Option::is_none"` keeps the wire
// shape identical to pre-C-B for callers that do not opt in. Existing call
// sites (SidecarGroundingClient::ground / synthesize_text) build the struct
// without a `.response_format(..)` call and are unaffected.
#[derive(Serialize)]
struct CompletionsRequest<'a> {
    prompt: &'a str,
    max_tokens: u32,
    temperature: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<serde_json::Value>,
}

impl<'a> CompletionsRequest<'a> {
    /// Construct with no constrained-decoding schema (pre-C-B shape).
    /// Internal helper; `response_format` defaults to `None`. Callers that
    /// want to opt in chain `.response_format(value)` after constructing.
    fn new(prompt: &'a str, max_tokens: u32, temperature: f64) -> Self {
        Self {
            prompt,
            max_tokens,
            temperature,
            response_format: None,
        }
    }

    /// Track C-B builder: attach a JSON-schema response_format payload. The
    /// caller is responsible for shaping `value` per the Anthropic /
    /// OpenAI-compatible spec:
    ///   { "type": "json_schema",
    ///     "json_schema": { "name": "...", "schema": { ... } } }
    /// The sidecar warns-and-ignores malformed shapes (returns unconstrained
    /// output), so a misshaped value here will not 4xx — it will silently
    /// disable constraint. Tests must assert on the parsed body.
    #[allow(dead_code)] // wired through by future tracks; exposed for tests
    fn response_format(mut self, value: serde_json::Value) -> Self {
        self.response_format = Some(value);
        self
    }
}

#[derive(Deserialize)]
struct CompletionsChoice {
    text: String,
}

#[derive(Deserialize)]
struct CompletionsResponse {
    choices: Vec<CompletionsChoice>,
}

impl GroundingClient for SidecarGroundingClient {
    async fn ground(
        &self,
        prompt: &str,
        level: ResearchLevel,
    ) -> Result<ResearchReport, GroundingError> {
        let url = self.completions_url();
        // Track C-B: callers opt into constrained-JSON via .response_format(..)
        // on this builder. Default (no chained call) keeps the pre-C-B wire
        // shape — `response_format` is `None` and `skip_serializing_if` omits
        // it from the serialized request body.
        let rendered = level.render(prompt);
        let body = CompletionsRequest::new(
            &rendered,
            GROUNDING_MAX_TOKENS,
            GROUNDING_TEMPERATURE,
        );

        debug!(level = ?level, url = %url, "sidecar_grounding_request");

        let response = match self
            .http
            .post(&url)
            .timeout(self.timeout)
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) if e.is_timeout() => {
                warn!(error = %e, "sidecar_grounding_timeout");
                return Err(GroundingError::Timeout {
                    url,
                    after: self.timeout,
                });
            }
            Err(e) if e.is_connect() => {
                warn!(error = %e, "sidecar_grounding_unavailable");
                return Err(GroundingError::SidecarUnavailable { url });
            }
            Err(e) => {
                warn!(error = %e, "sidecar_grounding_error");
                return Err(GroundingError::RequestFailed {
                    reason: e.to_string(),
                });
            }
        };

        let status = response.status();
        if !status.is_success() {
            return Err(GroundingError::HttpStatus {
                status: status.as_u16(),
                url,
            });
        }

        let parsed: CompletionsResponse = response.json().await.map_err(|e| {
            GroundingError::InvalidJson {
                reason: format!("completions envelope: {e}"),
            }
        })?;

        let text = parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.text)
            .ok_or(GroundingError::EmptyResponse)?;

        parse_research_report_json(&text)
    }

    async fn synthesize_text(&self, prompt: &str) -> Result<String, GroundingError> {
        let url = self.completions_url();
        // Send prompt verbatim — no template wrapping.
        // Track C-B: synthesize_text intentionally does NOT opt into
        // constrained decoding — synthesis output is free-form prose by
        // contract. Stays on the pre-C-B wire shape.
        let body = CompletionsRequest::new(prompt, GROUNDING_MAX_TOKENS, GROUNDING_TEMPERATURE);

        debug!(url = %url, "sidecar_synthesis_request");

        let response = match self
            .http
            .post(&url)
            .timeout(self.timeout)
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) if e.is_timeout() => {
                warn!(error = %e, "sidecar_synthesis_timeout");
                return Err(GroundingError::Timeout {
                    url,
                    after: self.timeout,
                });
            }
            Err(e) if e.is_connect() => {
                warn!(error = %e, "sidecar_synthesis_unavailable");
                return Err(GroundingError::SidecarUnavailable { url });
            }
            Err(e) => {
                warn!(error = %e, "sidecar_synthesis_error");
                return Err(GroundingError::RequestFailed {
                    reason: e.to_string(),
                });
            }
        };

        let status = response.status();
        if !status.is_success() {
            return Err(GroundingError::HttpStatus {
                status: status.as_u16(),
                url,
            });
        }

        let parsed: CompletionsResponse = response.json().await.map_err(|e| {
            GroundingError::InvalidJson {
                reason: format!("synthesis envelope: {e}"),
            }
        })?;

        parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.text)
            .ok_or(GroundingError::EmptyResponse)
    }
}

// ---------------------------------------------------------------------------
// SyntheticGroundingClient — deterministic stub. Intended for tests and the
// `ExecutionMode::Deterministic` path; never a silent production fallback.
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct SyntheticGroundingClient;

impl SyntheticGroundingClient {
    pub fn new() -> Self {
        Self
    }

    pub fn build_report_for_test(&self, topic: &str) -> ResearchReport {
        synthetic_report(topic)
    }
}

impl GroundingClient for SyntheticGroundingClient {
    async fn ground(
        &self,
        prompt: &str,
        _level: ResearchLevel,
    ) -> Result<ResearchReport, GroundingError> {
        Ok(synthetic_report(prompt))
    }

    async fn synthesize_text(&self, prompt: &str) -> Result<String, GroundingError> {
        // Deterministic stub: return empty string so Phase F falls back to deterministic path.
        // Tests for Phase F verify the fallback behavior, not LLM synthesis.
        let _ = prompt;
        Err(GroundingError::EmptyResponse)
    }
}

// ---------------------------------------------------------------------------
// CannedSynthesisGroundingClient — test fixture that returns a pre-configured
// cited-prose string from synthesize_text, allowing happy-path coverage of the
// LLM-success branch in phase_f_synthesize.  The ground() method delegates to
// SyntheticGroundingClient so existing tree tests are unaffected.
// ---------------------------------------------------------------------------

/// A test-only GroundingClient that returns `canned` from `synthesize_text`.
/// Construct via `CannedSynthesisGroundingClient::new(s)`.
/// Intended for happy-path coverage of Phase F synthesis in grounding_client_contract tests.
#[derive(Debug, Clone)]
pub struct CannedSynthesisGroundingClient {
    canned: String,
    inner: SyntheticGroundingClient,
}

impl CannedSynthesisGroundingClient {
    pub fn new(canned: String) -> Self {
        Self {
            canned,
            inner: SyntheticGroundingClient::new(),
        }
    }
}

impl GroundingClient for CannedSynthesisGroundingClient {
    async fn ground(
        &self,
        prompt: &str,
        level: ResearchLevel,
    ) -> Result<ResearchReport, GroundingError> {
        self.inner.ground(prompt, level).await
    }

    async fn synthesize_text(&self, _prompt: &str) -> Result<String, GroundingError> {
        Ok(self.canned.clone())
    }
}

fn synthetic_report(topic: &str) -> ResearchReport {
    ResearchReport {
        question: topic.to_owned(),
        answer: format!("Synthetic answer for: {topic}"),
        alternatives: vec![Alternative {
            name: format!("{topic}-approach-a"),
            description: format!("First approach for {topic}"),
            citations: vec![],
            confidence: 0.7,
            trade_offs: vec!["complexity".into()],
        }],
        contradictions: Vec::new(),
        sub_results: vec![SubQuestionResult {
            question: format!("What concept applies to {topic}?"),
            answer: format!("concept analysis for {topic}"),
            citations: vec![],
            confidence: 0.8,
            evidence_level: EvidenceLevel::MultiSource,
        }],
        snapshot: ResearchTreeSnapshot {
            nodes: Vec::new(),
            convergence: ConvergenceStatus::Converged,
            total_iterations: 1,
            elapsed: Duration::ZERO,
        },
        quality_metrics: Vec::new(),
        convergence: ConvergenceStatus::Converged,
    }
}

// ---------------------------------------------------------------------------
// parse_research_report_json — tolerant parser for LLM JSON output. Strips an
// optional ```json ... ``` fence and maps parse errors to GroundingError.
// ---------------------------------------------------------------------------

pub fn parse_research_report_json(raw: &str) -> Result<ResearchReport, GroundingError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(GroundingError::EmptyResponse);
    }

    let inner = strip_code_fence(trimmed);
    serde_json::from_str::<ResearchReport>(inner).map_err(|e| GroundingError::InvalidJson {
        reason: e.to_string(),
    })
}

fn strip_code_fence(s: &str) -> &str {
    let s = s.trim();
    let stripped = s
        .strip_prefix("```json")
        .or_else(|| s.strip_prefix("```"))
        .unwrap_or(s);
    let stripped = stripped.trim_start_matches('\n').trim();
    stripped.strip_suffix("```").unwrap_or(stripped).trim()
}

// ---------------------------------------------------------------------------
// Unit tests — template rendering and fence stripping.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_name_is_stable() {
        assert_eq!(ResearchLevel::Idea.template_name(), "l0_idea");
        assert_eq!(ResearchLevel::Concept.template_name(), "l1_concept");
        assert_eq!(ResearchLevel::Phase.template_name(), "l2_phase");
        assert_eq!(ResearchLevel::Step.template_name(), "l3_step");
    }

    #[test]
    fn template_renders_prompt_substitution() {
        let rendered = ResearchLevel::Idea.render("build a thing");
        assert!(rendered.contains("build a thing"));
        assert!(!rendered.contains("{prompt}"));
    }

    #[test]
    fn strip_code_fence_removes_json_wrapper() {
        let raw = "```json\n{\"a\": 1}\n```";
        assert_eq!(strip_code_fence(raw), "{\"a\": 1}");
    }

    #[test]
    fn strip_code_fence_removes_bare_wrapper() {
        let raw = "```\n{\"a\": 1}\n```";
        assert_eq!(strip_code_fence(raw), "{\"a\": 1}");
    }

    #[test]
    fn strip_code_fence_passes_through_plain_json() {
        assert_eq!(strip_code_fence("{\"a\": 1}"), "{\"a\": 1}");
    }

    #[test]
    fn default_sidecar_targets_port_8100() {
        let c = SidecarGroundingClient::default();
        assert!(c.base_url().contains("8100"));
    }
}
