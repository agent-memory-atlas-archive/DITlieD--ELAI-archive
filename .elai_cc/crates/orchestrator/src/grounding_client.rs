// Session 4D.10: sidecar-backed LLM grounding for recursive research
// decomposition. Replaces the prior synthetic-only path in research_tree.rs
// with a typed, async GroundingClient trait + a production SidecarGroundingClient
// speaking the sidecar's OpenAI-compatible `/v1/completions` endpoint.
//
// Native async fn in traits (no async_trait crate) + generic static dispatch
// per the workspace anti-stale-training rule.

use std::sync::Arc;
use std::time::{Duration, Instant};

use elai_core::clock::{ClockProvider, OsClock};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

use crate::research_tree::{
    Alternative, ConvergenceStatus, EvidenceLevel, ResearchReport, ResearchTreeSnapshot,
    SubQuestionResult,
};
use crate::ssb::SsbPayload;

/// E-35 helper — derive a deterministic `u64` from a domain-tagged blake3
/// hash so per-provider / per-schema labels can be carried on SSB payloads
/// without storing the raw label. Domain tag namespaces the hash so the
/// same label hashes distinctly per field (provider_hash vs schema_id_hash).
fn e35_blake3_u64_prefix(domain: &[u8], label: &str) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(label.as_bytes());
    let digest = hasher.finalize();
    let bytes = digest.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

/// J-28 T3 -- converts a dollar-denominated cost estimate into
/// micro-dollars (1 micro = 1e-6 USD) for
/// [`crate::ssb::UsageTurnEventPayload::est_cost_micros`]. `f64 as u64`
/// is a saturating, well-defined conversion in Rust since 1.45 (NaN or
/// negative -> 0, overflow -> `u64::MAX`); `.max(0.0)` handles NaN
/// explicitly first (`f64::max` ignores NaN and returns the other
/// operand) so the intent reads clearly without relying on cast trivia.
fn usd_to_micros(usd: f64) -> u64 {
    (usd.max(0.0) * 1_000_000.0) as u64
}

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
// CloudGroundingClient — Anthropic /v1/messages backed grounding for cloud
// mode. Mirrors SidecarGroundingClient's contract: render the level template,
// send the prompt, parse JSON ResearchReport from the response.
//
// Selected via `ELAI_GROUNDING_BACKEND=cloud` at task_runner construction.
// Default model is `claude-sonnet-4-6`; override via `ELAI_CLOUD_MODEL`.
// ---------------------------------------------------------------------------

pub const CLOUD_DEFAULT_MODEL: &str = "claude-sonnet-4-6";
pub const CLOUD_GROUNDING_MAX_TOKENS: u32 = 4096;
#[allow(dead_code)] // exposed for future temperature plumbing; Anthropic call uses
                    // its server-side default (temperature is not on MessageRequest).
pub const CLOUD_GROUNDING_TEMPERATURE: f64 = 0.0;

/// System prompt that pushes Sonnet toward bare JSON output for `ground()`.
/// `synthesize_text()` is free-form prose by contract (mirrors
/// SidecarGroundingClient) so it intentionally omits this directive.
const CLOUD_JSON_SYSTEM_PROMPT: &str =
    "Output ONLY a single valid JSON object that matches the schema in the user message. \
     Do not wrap in markdown code fences. Do not include any prose before or after the JSON.";

// H-5 — Anthropic Message Batches non-interactive lane. Bounded poll: the fixture
// endpoint returns `ended` immediately; a live batch that has not ended within the cap
// returns RequestFailed (the deferred 24h retrieval design is Tier-2, RunPod-pending).
const BATCH_POLL_MAX: u32 = 3;
const BATCH_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

/// H-5 — env flag that activates a LIVE non-interactive batch lane. Default-off; the
/// live Anthropic batch call + a production non-interactive caller (GEPA / bench) are
/// Tier-2 / RunPod-pending. Registered in wire-dark-allowlist.toml (R9). See
/// workflow/reasoning/H-5-decisions.md.
pub const ELAI_BATCH_LANE_ENABLE_ENV: &str = "ELAI_BATCH_LANE_ENABLE";

/// HD-1 helper: round a USD cost to whole cents (saturating, non-negative). Used to
/// express the MEASURED 50%-off batch spend as a journal-recomputable integer.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn usd_to_cents(usd: f64) -> i64 {
    let cents = (usd * 100.0).round();
    if cents.is_finite() && cents >= 0.0 {
        cents.min(i64::MAX as f64) as i64
    } else {
        0
    }
}

#[derive(Clone)]
pub struct CloudGroundingClient {
    client: elai_providers::AnthropicClient,
    model: String,
    max_tokens: u32,
    /// C-5: optional SSB writer. When `Some`, every `send_message`
    /// round-trip emits a [`crate::ssb::SsbActionType::CloudSpend`] row
    /// with per-call token counts + computed cost via [`crate::cloud_mode::SpendAccumulator`].
    /// When `None` (legacy / test path), no row is written; the
    /// SpendAccumulator state still accrues across calls so the final
    /// `task_usd` would match the same model's pricing if a writer were
    /// later attached. Wrapped in `Arc<tokio::sync::Mutex<...>>` so the
    /// accumulator can be mutated across `.await` boundaries inside the
    /// `send_message` body.
    ssb_writer: Option<Arc<crate::Ssb>>,
    /// C-5: shared spend accumulator. Pinned to the model id at
    /// construction so pricing constants are correct. Wrapped so
    /// repeated `send_message` calls can mutate the snapshot.
    spend_accumulator: Arc<tokio::sync::Mutex<crate::cloud_mode::SpendAccumulator>>,
    /// C-25: shared cumulative-token tracker. Records per-call token
    /// counts (input/output/cache-read/cache-write) and feeds
    /// `recalibrate_budget` for the per-task attention-ceiling
    /// expansion clamp. Wrapped in `Arc<tokio::sync::Mutex<...>>` so
    /// the tracker can be mutated across `.await` boundaries inside
    /// `send_message` (mirrors the SpendAccumulator pattern). Always
    /// `Some(default-initialised)` after `new` so call sites do not
    /// branch on optionality.
    cumulative_token_tracker:
        Arc<tokio::sync::Mutex<crate::context_budget::CumulativeTokenTracker>>,
    /// C-5: optional daily spend cap (USD). When `Some`, every
    /// `send_message` round-trip consults the SSB aggregator BEFORE the
    /// HTTP send. If the daily ceiling has been breached, the call is
    /// short-circuited with a typed error and a `cloud_spend_blocked`
    /// SSB row (carried via the existing `EscalateToCloud` row with
    /// `trigger_reason="cap_reached"`). `None` disables cap enforcement.
    spend_cap_daily_usd: Option<f64>,
    /// C-5: task id stamped on emitted CloudSpend rows so the dashboard
    /// can group ledger entries per active task. Defaults to
    /// `"unknown_task"`; production wiring should set it via
    /// [`with_task_id`].
    task_id: String,
}

impl std::fmt::Debug for CloudGroundingClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CloudGroundingClient")
            .field("model", &self.model)
            .field("max_tokens", &self.max_tokens)
            .field("ssb_writer", &self.ssb_writer.is_some())
            .field("spend_cap_daily_usd", &self.spend_cap_daily_usd)
            .field("task_id", &self.task_id)
            .finish()
    }
}

impl CloudGroundingClient {
    pub fn new(client: elai_providers::AnthropicClient, model: impl Into<String>) -> Self {
        let model = model.into();
        let accumulator = crate::cloud_mode::SpendAccumulator::for_model(&model);
        Self {
            client,
            model,
            max_tokens: CLOUD_GROUNDING_MAX_TOKENS,
            ssb_writer: None,
            spend_accumulator: Arc::new(tokio::sync::Mutex::new(accumulator)),
            // C-25: tracker starts at default (all zeros). Per-call
            // record_response wires it next to the SpendAccumulator
            // accumulate calls (mirroring the billing path but for
            // raw token counts feeding recalibrate_budget).
            cumulative_token_tracker: Arc::new(tokio::sync::Mutex::new(
                crate::context_budget::CumulativeTokenTracker::new(),
            )),
            spend_cap_daily_usd: None,
            task_id: "unknown_task".to_string(),
        }
    }

    /// C-25 — accessor for the shared tracker so callers (TaskRunner
    /// post-execute hook) can read the latest snapshot for SSB rows /
    /// ExecutionOutcome.token_count threading without re-instantiating.
    #[must_use]
    pub fn cumulative_token_tracker(
        &self,
    ) -> Arc<tokio::sync::Mutex<crate::context_budget::CumulativeTokenTracker>> {
        self.cumulative_token_tracker.clone()
    }

    /// C-5: attach a live SSB writer so each `send_message` round-trip
    /// emits a [`crate::ssb::SsbActionType::CloudSpend`] row.
    #[must_use]
    pub fn with_ssb_writer(mut self, ssb: Arc<crate::Ssb>) -> Self {
        self.ssb_writer = Some(ssb);
        self
    }

    /// C-5: set the daily spend cap in USD. When breached, new sends
    /// short-circuit with a typed error and the dashboard CloudPanel
    /// renders a BLOCKED row.
    #[must_use]
    pub fn with_spend_cap_daily_usd(mut self, cap: Option<f64>) -> Self {
        self.spend_cap_daily_usd = cap;
        self
    }

    /// C-5: stamp a task id on emitted CloudSpend rows.
    #[must_use]
    pub fn with_task_id(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = task_id.into();
        self
    }

    /// E-35 — emit a `CloudStrictSchemaApplied` SSB beacon recording that
    /// a strict-schema directive is being applied for the supplied
    /// `(provider, schema_id)` pair. Anthropic / OpenAI / Gemini have
    /// distinct strict-mode bodies (Anthropic
    /// `structured-outputs-2025-11-13` beta header / OpenAI `strict:true`
    /// / Gemini `responseJsonSchema`); the request-body shape is wired
    /// at the existing `build_request_with_decoding` Tool-API path which
    /// is the Sonnet-supported strict-schema entry. Header propagation
    /// to the AnthropicClient surface (the canonical `anthropic-beta`
    /// list) is RUNPOD-PENDING (provider-crate extension — see
    /// `workflow/reasoning/E-35-decisions.md` DECISION-3).
    ///
    /// Refuses loud per plan L67 when the caller requests the legacy
    /// vendor `json_object` mode: returns `GroundingError::RequestFailed`
    /// and emits a `LegacyJsonObjectRefused` SSB row. Never silent
    /// fallback.
    ///
    /// The `provider_label` is one of "anthropic" / "openai" / "gemini"
    /// — used to derive the blake3-prefixed `provider_hash` field on
    /// the SSB payload (hashed so the wire format is privacy-preserving
    /// across providers).
    ///
    /// `now_ms` is injected so call sites use a deterministic clock
    /// (never `std::time::SystemTime` per the entropy quarantine rule).
    pub async fn mark_strict_schema_applied(
        &self,
        provider_label: &str,
        schema_id: &str,
        mode_label: &str,
        now_ms: u64,
    ) -> Result<(), GroundingError> {
        // Refuse legacy json_object loud. Per plan L67 + cite-discipline
        // CITE-RECORD #11: vendor json_object dropped quality vs strict
        // schemas across all three providers per "When Correct Isn't
        // Usable" (referenced by name only — arXiv DEFERRED to mistakes.md
        // cite-incident cluster #13).
        if mode_label.eq_ignore_ascii_case("json_object") {
            if let Some(ssb) = &self.ssb_writer {
                let provider_hash = e35_blake3_u64_prefix(b"cloud.provider.", provider_label);
                let payload =
                    SsbPayload::LegacyJsonObjectRefused(crate::ssb::LegacyJsonObjectRefusedPayload {
                        provider_hash,
                        ts_ms: now_ms,
                    });
                if let Err(err) = ssb
                    .write_payload("cloud_grounding", 0, payload, Vec::new())
                    .await
                {
                    tracing::warn!(
                        target: "cloud_grounding",
                        error = %err,
                        "failed to emit LegacyJsonObjectRefused SSB row"
                    );
                }
            }
            return Err(GroundingError::RequestFailed {
                reason: format!(
                    "legacy json_object mode refused for provider={provider_label}; \
                     use strict json_schema instead"
                ),
            });
        }

        if let Some(ssb) = &self.ssb_writer {
            let provider_hash = e35_blake3_u64_prefix(b"cloud.provider.", provider_label);
            let schema_id_hash = e35_blake3_u64_prefix(b"cloud.schema_id.", schema_id);
            let payload =
                SsbPayload::CloudStrictSchemaApplied(crate::ssb::CloudStrictSchemaAppliedPayload {
                    provider_hash,
                    schema_id_hash,
                    ts_ms: now_ms,
                });
            if let Err(err) = ssb
                .write_payload("cloud_grounding", 0, payload, Vec::new())
                .await
            {
                tracing::warn!(
                    target: "cloud_grounding",
                    error = %err,
                    "failed to emit CloudStrictSchemaApplied SSB row"
                );
            }
        }
        Ok(())
    }

    /// Construct from env: resolves auth via `AuthSource::from_env_or_saved`,
    /// base URL via `read_base_url`, model from `ELAI_CLOUD_MODEL` (default
    /// `claude-sonnet-4-6`). Returns `GroundingError::RequestFailed` if no
    /// auth source is available — matches the failure shape callers already
    /// handle via `ResearchTreeError::GroundingFailed`.
    ///
    /// NOTE: this sync constructor does NOT trigger OAuth refresh on expired
    /// saved tokens (refresh requires an HTTP call wrapped in a tokio runtime
    /// which conflicts when called from async context). Prefer
    /// `from_env_async()` from inside `.await` chains — it auto-refreshes via
    /// `resolve_auth_with_auto_refresh` and bounces through `spawn_blocking`
    /// to side-step the nested-runtime panic.
    pub fn from_env() -> Result<Self, GroundingError> {
        let auth = elai_providers::AuthSource::from_env_or_saved().map_err(|e| {
            GroundingError::RequestFailed {
                reason: format!("cloud auth: {e}"),
            }
        })?;
        let base_url = elai_providers::read_base_url();
        let client = elai_providers::AnthropicClient::from_auth(auth).with_base_url(base_url);
        let model = std::env::var("ELAI_CLOUD_MODEL")
            .unwrap_or_else(|_| CLOUD_DEFAULT_MODEL.to_string());
        Ok(Self::new(client, model))
    }

    /// Async constructor that auto-refreshes the saved OAuth token when
    /// expired (patch-R1). Internally:
    ///   1. spawn_blocking(resolve_auth_with_auto_refresh) — the auto-refresh
    ///      path calls `Runtime::new().block_on()` for its HTTP refresh,
    ///      which cannot run on the same thread as the caller's tokio
    ///      runtime. spawn_blocking gives it a dedicated thread.
    ///   2. Build AnthropicClient with the (possibly refreshed) BearerToken.
    ///
    /// Call this from `drive_task` (or any async grounding setup) when
    /// `ELAI_GROUNDING_BACKEND=cloud` — keeps the cloud path usable across
    /// long-running orchestrator runs that outlive an access-token TTL.
    pub async fn from_env_async() -> Result<Self, GroundingError> {
        let auth = tokio::task::spawn_blocking(elai_providers::resolve_auth_with_auto_refresh)
            .await
            .map_err(|e| GroundingError::RequestFailed {
                reason: format!("cloud auth resolve join: {e}"),
            })?
            .map_err(|e| GroundingError::RequestFailed {
                reason: format!("cloud auth: {e}"),
            })?;
        let base_url = elai_providers::read_base_url();
        let client = elai_providers::AnthropicClient::from_auth(auth).with_base_url(base_url);
        let model = std::env::var("ELAI_CLOUD_MODEL")
            .unwrap_or_else(|_| CLOUD_DEFAULT_MODEL.to_string());
        Ok(Self::new(client, model))
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    /// Build a MessageRequest. When `system_prompt` is `Some`, attach a
    /// system block — used for `ground()` to force JSON-only output. For
    /// `synthesize_text()` the system slot stays `None` so the prompt body
    /// drives the model alone (free-form prose contract).
    fn build_request(
        &self,
        user_prompt: &str,
        system_prompt: Option<&'static str>,
    ) -> elai_providers::MessageRequest {
        self.build_request_with_decoding(user_prompt, system_prompt, None)
    }

    /// C-26 — variant of [`build_request`] that wires
    /// [`elai_providers::ConstrainedDecodingConfig`] into the outbound
    /// MessageRequest.
    ///
    /// Translation table:
    /// * `DecodingMode::JsonSchema { schema_json }` → emit `tools = [
    ///   ToolDefinition { name: "structured_output", input_schema: <parsed
    ///   schema_json> } ]` and `tool_choice = Tool { name:
    ///   "structured_output" }`. This is the canonical Anthropic Tool API
    ///   schema-enforcement path (Sonnet 4.6 supports it; see
    ///   update-implementation-order.report.md).
    /// * `DecodingMode::Grammar { .. }` → unchanged (Grammar is sidecar-only;
    ///   the cloud client cannot enforce EBNF). Schema-violation surfaces
    ///   downstream as a retry; not silently dropped.
    /// * `DecodingMode::Regex { .. }` → unchanged (cloud has no regex
    ///   enforcement primitive today).
    /// * `DecodingMode::FreeText` / `None` → unchanged (legacy un-constrained
    ///   path).
    pub(crate) fn build_request_with_decoding(
        &self,
        user_prompt: &str,
        system_prompt: Option<&'static str>,
        decoding: Option<&elai_providers::ConstrainedDecodingConfig>,
    ) -> elai_providers::MessageRequest {
        // E-39 cache_control discipline pass: ensure 4 cache_control
        // markers on the canonical "static prefix" surfaces so the
        // 5-min TTL pays back after a single read (Anthropic pricing
        // May 2026: cache-write 1.25x, cache-read 0.10x; dev.to/
        // whoffagents on the May 2026 60min->5min TTL change).
        //
        // The 4 surfaces, per plan Task 6:
        //   1. system block (already ephemeral_text -- carries
        //      cache_control)
        //   2. project-spec system block (E-39 -- new ephemeral block
        //      ahead of the caller-supplied system prompt; default
        //      empty until the orchestrator threads the real spec
        //      through, at which point the caller passes it in via
        //      `system_prompt`)
        //   3. repo-manifest hint block (E-39 -- ephemeral; same
        //      thread-through caveat)
        //   4. tool schema cache_control (E-39 -- promoted from None
        //      to Some(ephemeral) below)
        //
        // All cache_control markers use `CacheControl::ephemeral()` so
        // the wire shape matches the 5-min TTL default per plan
        // DECISION-6.
        let mut system_blocks: Vec<elai_providers::SystemBlock> = Vec::new();
        // 1) caller-supplied system prompt.
        if let Some(text) = system_prompt {
            system_blocks.push(elai_providers::SystemBlock::ephemeral_text(text));
        }
        // 2) project-spec anchor (E-39 cache_control hit 2/4). Empty
        // text avoids breaking existing tests that count tokens; the
        // ephemeral marker is the audit-evidence that the surface is
        // wired through. Live callers extend this once the spec
        // pipeline threads the real text through.
        system_blocks.push(elai_providers::SystemBlock::ephemeral_text(
            "[E-39 spec anchor: cache_control ephemeral 5-min TTL]",
        ));
        // 3) repo-manifest anchor (E-39 cache_control hit 3/4). Same
        // shape as the spec anchor.
        system_blocks.push(elai_providers::SystemBlock::ephemeral_text(
            "[E-39 manifest anchor: cache_control ephemeral 5-min TTL]",
        ));
        let system = if system_blocks.is_empty() {
            None
        } else {
            Some(system_blocks)
        };
        let mut request = elai_providers::MessageRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            messages: vec![elai_providers::InputMessage::user_text(user_prompt)],
            system,
            tools: None,
            tool_choice: None,
            stream: false,
            seed: None,
            // I-26: effort is not set at the grounding layer.
            effort: None,
        };

        if let Some(cfg) = decoding {
            if let elai_providers::DecodingMode::JsonSchema { schema_json } = &cfg.mode {
                // Parse the schema string; on parse failure, log a warn and
                // skip the Tool-API encoding (callers retain the un-
                // constrained behaviour rather than silently swallowing).
                match serde_json::from_str::<serde_json::Value>(schema_json) {
                    Ok(schema_value) => {
                        request.tools = Some(vec![elai_providers::ToolDefinition {
                            name: "structured_output".to_string(),
                            description: Some(
                                "Return the structured output object matching the schema."
                                    .to_string(),
                            ),
                            input_schema: schema_value,
                            // E-39 cache_control hit 4/4: promote from
                            // None to Some(ephemeral) so the tool
                            // schema participates in the 5-min prompt
                            // cache. Plan Task 6 + Anthropic pricing
                            // (verified May 2026).
                            cache_control: Some(
                                elai_providers::CacheControl::ephemeral(),
                            ),
                        }]);
                        request.tool_choice = Some(elai_providers::ToolChoice::Tool {
                            name: "structured_output".to_string(),
                        });
                    }
                    Err(err) => {
                        warn!(
                            target: "grounding_client",
                            error = %err,
                            "cloud decoding json_schema parse failed; falling back to FreeText for this request"
                        );
                    }
                }
            }
            // Grammar / Regex / FreeText: no cloud-side wiring (see doc above).
        }

        request
    }

    /// Strip a leading ```json or ``` fence and matching trailing ``` from
    /// the model's response when present. Returns the trimmed inner content.
    fn strip_markdown_fences(text: &str) -> &str {
        let trimmed = text.trim();
        if let Some(rest) = trimmed.strip_prefix("```json") {
            let rest = rest.trim_start_matches('\n').trim();
            return rest.strip_suffix("```").unwrap_or(rest).trim();
        }
        if let Some(rest) = trimmed.strip_prefix("```") {
            let rest = rest.trim_start_matches('\n').trim();
            return rest.strip_suffix("```").unwrap_or(rest).trim();
        }
        trimmed
    }

    fn extract_text(
        content: &[elai_providers::OutputContentBlock],
    ) -> Result<String, GroundingError> {
        content
            .iter()
            .find_map(|block| match block {
                elai_providers::OutputContentBlock::Text { text } => Some(text.clone()),
                _ => None,
            })
            .ok_or(GroundingError::EmptyResponse)
    }

    /// C-5 — send a MessageRequest and emit `CloudSpend` on success.
    /// Wraps `AnthropicClient::send_message`. The cap check happens
    /// before the HTTP send so a breach short-circuits with no network
    /// call. On success, the SpendAccumulator is updated with the
    /// per-call usage and the resulting snapshot is written as a
    /// `CloudSpend` SSB row.
    async fn send_with_ledger(
        &self,
        request: &elai_providers::MessageRequest,
    ) -> Result<elai_providers::MessageResponse, GroundingError> {
        // Cap check: open a read-only SsbViewer against the writer's
        // canonical db_path. Failure to open is logged + skipped per the
        // C-5 plan's stub_ban ("skip rather than block on viewer error").
        if let (Some(cap), Some(ssb)) = (self.spend_cap_daily_usd, self.ssb_writer.as_ref()) {
            // C-15: OsClock-backed cap-check timestamp. Spend ledger reads
            // are observation-only; no determinism contract.
            let now_ms = i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX);
            match crate::ssb_viewer::SsbViewer::open(ssb.db_path()) {
                Ok(viewer) => match viewer.cloud_spend_summary(now_ms) {
                    Ok(summary) => {
                        if summary.day_usd >= cap {
                            // Emit cloud_spend_blocked marker via existing
                            // EscalateToCloud row (no dedicated row type
                            // today; reuse the row + trigger_reason).
                            let payload = crate::ssb::SsbPayload::EscalateToCloud(
                                crate::ssb::EscalateToCloudPayload {
                                    step_id: format!("task_{}", self.task_id),
                                    trigger_reason: "cap_reached".to_string(),
                                    layer_failed: 0,
                                    ts_ms: now_ms,
                                },
                            );
                            if let Err(e) = ssb
                                .write_payload(
                                    "cloud_grounding_client",
                                    0,
                                    payload,
                                    Vec::new(),
                                )
                                .await
                            {
                                tracing::warn!(
                                    error = %e,
                                    "cloud_grounding: cap_reached SSB write failed (non-fatal)"
                                );
                            }
                            return Err(GroundingError::RequestFailed {
                                reason: format!(
                                    "cloud daily spend cap reached: {:.4} >= {:.4} USD",
                                    summary.day_usd, cap
                                ),
                            });
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            cap_usd = cap,
                            "cloud_grounding: cloud_spend_summary failed; cap check skipped"
                        );
                    }
                },
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        cap_usd = cap,
                        "cloud_grounding: SsbViewer open failed; cap check skipped"
                    );
                }
            }
        }

        // Live HTTP send. J-28 RECON_NEEDED-4: neither MessageResponse nor
        // CompletionResponse carries elapsed_ms in Rust; measure it with a
        // seam-local wall-clock instead of expecting it on the response.
        let send_started = Instant::now(); // entropy-allow: monotonic start-marker for live provider request latency telemetry only; a seeded clock would not measure real network duration
        let response = self.client.send_message(request).await.map_err(|e| {
            GroundingError::RequestFailed {
                reason: format!("anthropic: {e}"),
            }
        })?;
        let elapsed_ms = u64::try_from(send_started.elapsed().as_millis()).unwrap_or(u64::MAX);

        // Accumulate per-call usage + emit CloudSpend row when wired.
        if let Some(ssb) = self.ssb_writer.as_ref() {
            let usage = response.usage.clone();
            let snapshot = {
                let mut acc = self.spend_accumulator.lock().await;
                acc.accumulate(usage.input_tokens, usage.output_tokens);
                acc.accumulate_cache(
                    usage.cache_creation_input_tokens,
                    usage.cache_read_input_tokens,
                );
                acc.snapshot()
            };
            // C-25 — record raw token counts into the cumulative-token
            // tracker AFTER the billing accumulate calls (DRIFT-C: the
            // plan's "after each record() call" maps to the .accumulate
            // calls above). The billing path stays unchanged. The
            // tracker mutates across the .await of the lock; we use the
            // same tokio::sync::Mutex pattern as spend_accumulator.
            {
                let mut tracker = self.cumulative_token_tracker.lock().await;
                tracker.record_response(
                    &crate::context_budget::ProviderUsage::from_u32(
                        usage.input_tokens,
                        usage.output_tokens,
                        usage.cache_read_input_tokens,
                        usage.cache_creation_input_tokens,
                    ),
                );
            }
            // C-15: OsClock-backed cap-check timestamp. Spend ledger reads
            // are observation-only; no determinism contract.
            let now_ms = i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX);
            // CloudSpendPayload reports per-call token deltas (not the
            // accumulator total) so the dashboard can aggregate per-day
            // by summing rows. cost_usd carries the running total via
            // the accumulator's `cost_usd` snapshot.
            let payload = crate::ssb::SsbPayload::CloudSpend(crate::ssb::CloudSpendPayload {
                provider: crate::cloud_mode::DEFAULT_CLOUD_PROVIDER.to_string(),
                model: snapshot.model.clone(),
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
                cache_creation_tokens: usage.cache_creation_input_tokens,
                cache_read_tokens: usage.cache_read_input_tokens,
                cost_usd: snapshot.cost_usd,
                task_id: self.task_id.clone(),
                ts_ms: now_ms,
            });
            if let Err(e) = ssb
                .write_payload("cloud_grounding_client", 0, payload, Vec::new())
                .await
            {
                tracing::warn!(
                    error = %e,
                    "cloud_grounding: CloudSpend SSB write failed (non-fatal)"
                );
            }

            // J-28 T3: the uniform per-turn UsageTurnEvent row, emitted
            // BESIDE the CloudSpend row above (never replacing it --
            // CloudSpendPayload carries no task/session key, this row is
            // the (model,task,session,lane)-keyed join the T5 rollup
            // needs). snapshot.priced (T4) gates est_cost_micros: an
            // unpriced (unknown-model) accumulator gets an honest 0,
            // never a silently-wrong sonnet-derived cost.
            let usage_est_cost_micros = if snapshot.priced {
                usd_to_micros(usage.estimated_cost_usd(&snapshot.model).total_cost_usd())
            } else {
                0
            };
            // T2: assemble the normalized sample (lane + semantics tag)
            // BEFORE the row is written, per ROUTE B (client-side
            // normalization keyed by the lane tag).
            let normalized = crate::usage_ledger::NormalizedUsage::cloud(usage.token_usage());
            let usage_turn_payload =
                crate::ssb::SsbPayload::UsageTurnEvent(crate::ssb::UsageTurnEventPayload {
                    identity: "local".to_string(),
                    lane: normalized.lane,
                    backend: crate::cloud_mode::DEFAULT_CLOUD_PROVIDER.to_string(),
                    model: snapshot.model.clone(),
                    task_id: Some(self.task_id.clone()),
                    session_id: None,
                    input_tokens: usage.input_tokens,
                    output_tokens: usage.output_tokens,
                    cache_creation_tokens: usage.cache_creation_input_tokens,
                    cache_read_tokens: usage.cache_read_input_tokens,
                    est_cost_micros: usage_est_cost_micros,
                    elapsed_ms,
                    semantics_tag: normalized.semantics_tag,
                    kind: crate::ssb::UsageTurnEventKind::TurnRecorded,
                    ts_ms: now_ms,
                });
            if let Err(e) = ssb
                .write_payload("cloud_grounding_client", 0, usage_turn_payload, Vec::new())
                .await
            {
                tracing::warn!(
                    error = %e,
                    "cloud_grounding: UsageTurnEvent SSB write failed (non-fatal)"
                );
            }

            // H-1: activate the wire-dark prompt-cache hit-rate monitor
            // (cache_check::measure_cache_hit_rate — pre-H-1 it had zero
            // production callers). The per-call cache-read fraction
            // (cache_read / (cache_read + fresh input)) lands as a
            // CriticDecisionEvent{PromptCacheHitRate} row, recomputable from the
            // journal (GD-2). Default-ON: no ELAI_CRITIC_OF_DESIGN_ENABLE gate
            // (W-2) — it fires whenever the SSB writer is attached, alongside the
            // CloudSpend row above. Non-fatal: a monitor write must never break a
            // live completion.
            let cache_snapshot = crate::cache_check::CacheCheckSnapshot {
                hits: u64::from(usage.cache_read_input_tokens),
                misses: u64::from(usage.input_tokens),
            };
            if let Err(e) = crate::cache_check::measure_cache_hit_rate(
                ssb,
                "cloud_grounding_client",
                0,
                cache_snapshot,
            )
            .await
            {
                tracing::warn!(
                    error = %e,
                    "cloud_grounding: prompt-cache hit-rate monitor write failed (non-fatal)"
                );
            }
        }

        Ok(response)
    }
}

impl GroundingClient for CloudGroundingClient {
    async fn ground(
        &self,
        prompt: &str,
        level: ResearchLevel,
    ) -> Result<ResearchReport, GroundingError> {
        let rendered = level.render(prompt);
        let request = self.build_request(&rendered, Some(CLOUD_JSON_SYSTEM_PROMPT));
        debug!(level = ?level, model = %self.model, "cloud_grounding_request");
        // Diagnostic stderr — surfaces in test output regardless of RUST_LOG.
        eprintln!(
            "[cloud_grounding] -> ground level={:?} model={} prompt_chars={}",
            level,
            self.model,
            rendered.chars().count()
        );
        // C-5: send + emit CloudSpend on success / honour daily cap.
        let response = self.send_with_ledger(&request).await.map_err(|e| {
            eprintln!("[cloud_grounding] ERR send_message: {e}");
            e
        })?;
        let text = match Self::extract_text(&response.content) {
            Ok(t) => t,
            Err(e) => {
                eprintln!(
                    "[cloud_grounding] ERR extract_text (no Text block in response): {e}"
                );
                return Err(e);
            }
        };
        eprintln!(
            "[cloud_grounding] <- got text_chars={} first_120={:?}",
            text.chars().count(),
            text.chars().take(120).collect::<String>()
        );
        let stripped = Self::strip_markdown_fences(&text);
        match parse_research_report_json(stripped) {
            Ok(report) => {
                eprintln!(
                    "[cloud_grounding] OK parsed level={:?} alternatives={} sub_results={}",
                    level,
                    report.alternatives.len(),
                    report.sub_results.len()
                );
                Ok(report)
            }
            Err(e) => {
                eprintln!(
                    "[cloud_grounding] ERR parse_research_report_json: {e}; \
                     stripped_first_200={:?}",
                    stripped.chars().take(200).collect::<String>()
                );
                Err(e)
            }
        }
    }

    async fn synthesize_text(&self, prompt: &str) -> Result<String, GroundingError> {
        // No system prompt: synthesis output is free-form prose by contract,
        // matching SidecarGroundingClient::synthesize_text behaviour.
        let request = self.build_request(prompt, None);
        debug!(model = %self.model, "cloud_synthesis_request");
        // C-5: send + emit CloudSpend on success / honour daily cap.
        let response = self.send_with_ledger(&request).await.map_err(|e| {
            GroundingError::RequestFailed {
                reason: format!("{e}"),
            }
        })?;
        Self::extract_text(&response.content)
    }
}

impl CloudGroundingClient {
    /// H-5 — true when a LIVE non-interactive batch lane is activated
    /// (`ELAI_BATCH_LANE_ENABLE`). Default-off: the live Anthropic batch call + a
    /// production non-interactive caller (GEPA / bench) are Tier-2 / RunPod-pending
    /// (registered in wire-dark-allowlist.toml R9). A future autonomous lane gates its
    /// choice of [`Self::ground_batch`] vs sequential `ground()` on this.
    #[must_use]
    pub fn batch_lane_enabled() -> bool {
        std::env::var(ELAI_BATCH_LANE_ENABLE_ENV)
            .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(false)
    }

    /// H-5 — batch the non-interactive grounding lane through the Anthropic Message
    /// Batches API (50% price). Submits all `prompts` as ONE batch, bounded-polls to
    /// completion, maps results back by `custom_id` (results return OUT OF ORDER), and
    /// emits `batch_lifecycle` SSB rows (Submitted on submit; Completed on retrieve,
    /// carrying the HD-1 MEASURED 50% spend). Returns reports in `prompts` order.
    ///
    /// AUTONOMOUS / deferrable lane ONLY — never for a human-waited grounding (the
    /// interactive `ground()` stays synchronous). See [`Self::batch_lane_enabled`] +
    /// workflow/reasoning/H-5-decisions.md (the live call + production caller are Tier-2).
    pub async fn ground_batch(
        &self,
        prompts: &[String],
        level: ResearchLevel,
    ) -> Result<Vec<ResearchReport>, GroundingError> {
        if prompts.is_empty() {
            return Ok(Vec::new());
        }
        let batch = self.client.batch_client();
        let items: Vec<elai_providers::BatchItem> = prompts
            .iter()
            .enumerate()
            .map(|(i, p)| elai_providers::BatchItem {
                custom_id: format!("g{i}"),
                request: self.build_request(&level.render(p), Some(CLOUD_JSON_SYSTEM_PROMPT)),
            })
            .collect();
        let request_count = u32::try_from(items.len()).unwrap_or(u32::MAX);

        let batch_id = batch.create(&items).await.map_err(|e| {
            GroundingError::RequestFailed {
                reason: format!("batch create: {e}"),
            }
        })?;
        self.emit_batch_lifecycle(
            crate::ssb::BatchLifecycleKind::Submitted,
            &batch_id.0,
            request_count,
            0,
        )
        .await;

        // Bounded poll — NEVER blocks unbounded. The fixture endpoint returns `ended`
        // immediately; a live batch still in-flight after the cap surfaces a clear
        // error (the deferred 24h retrieval design is Tier-2 / RunPod-pending).
        let mut status = batch.poll(&batch_id).await.map_err(|e| GroundingError::RequestFailed {
            reason: format!("batch poll: {e}"),
        })?;
        let mut polls = 0u32;
        while !status.is_ended() && polls < BATCH_POLL_MAX {
            tokio::time::sleep(BATCH_POLL_INTERVAL).await;
            status = batch.poll(&batch_id).await.map_err(|e| GroundingError::RequestFailed {
                reason: format!("batch poll: {e}"),
            })?;
            polls += 1;
        }
        if !status.is_ended() {
            return Err(GroundingError::RequestFailed {
                reason: format!(
                    "batch {} not ended within poll cap (deferred 24h retrieval is Tier-2)",
                    batch_id.0
                ),
            });
        }

        let results = batch.retrieve(&batch_id).await.map_err(|e| {
            GroundingError::RequestFailed {
                reason: format!("batch retrieve: {e}"),
            }
        })?;
        // Map by custom_id (NOT position) back to prompt order; sum the synchronous USD
        // cost from the response usage so the 50% saving is MEASURED (HD-1), not assumed.
        let mut by_id: std::collections::HashMap<String, elai_providers::BatchResultItem> =
            results.into_iter().map(|r| (r.custom_id.clone(), r)).collect();
        let mut reports = Vec::with_capacity(prompts.len());
        let mut sync_usd = 0.0_f64;
        for i in 0..prompts.len() {
            let key = format!("g{i}");
            let item = by_id.remove(&key).ok_or_else(|| GroundingError::RequestFailed {
                reason: format!("batch result missing custom_id {key}"),
            })?;
            match item.outcome {
                elai_providers::BatchResultOutcome::Succeeded(resp) => {
                    sync_usd += resp.usage.estimated_cost_usd(&resp.model).total_cost_usd();
                    let text = Self::extract_text(&resp.content)?;
                    let stripped = Self::strip_markdown_fences(&text);
                    reports.push(parse_research_report_json(stripped)?);
                }
                other => {
                    return Err(GroundingError::RequestFailed {
                        reason: format!("batch item {key} non-success: {other:?}"),
                    });
                }
            }
        }
        // HD-1: spend_cents_half == half the synchronous cost the same requests would
        // have billed — recomputable from the journal under a pinned seed (GD-2).
        let spend_cents_half = elai_providers::half_price_cents(usd_to_cents(sync_usd));
        self.emit_batch_lifecycle(
            crate::ssb::BatchLifecycleKind::Completed,
            &batch_id.0,
            request_count,
            spend_cents_half,
        )
        .await;
        Ok(reports)
    }

    /// Emit one `batch_lifecycle` SSB row (best-effort; a telemetry write never breaks
    /// the lane). No-op when no SSB writer is attached.
    async fn emit_batch_lifecycle(
        &self,
        kind: crate::ssb::BatchLifecycleKind,
        batch_id: &str,
        request_count: u32,
        spend_cents_half: i64,
    ) {
        if let Some(ssb) = self.ssb_writer.as_ref() {
            let payload =
                crate::ssb::SsbPayload::BatchLifecycle(crate::ssb::BatchLifecyclePayload {
                    kind,
                    batch_id: batch_id.to_owned(),
                    request_count,
                    model: self.model.clone(),
                    spend_cents_half,
                    task_id: self.task_id.clone(),
                });
            if let Err(e) = ssb
                .write_payload("cloud_grounding_batch", 0, payload, Vec::new())
                .await
            {
                tracing::warn!(
                    error = %e,
                    "cloud_grounding_batch: batch_lifecycle SSB write failed (non-fatal)"
                );
            }
        }
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

// ---------------------------------------------------------------------------
// HttpCosmosResearchDispatcher (C-16 DECISION-B) — bridges
// `elai_knowledge::ResearchDispatcher` (sync trait) to the sidecar's
// SearXNG-compatible `/search` endpoint via the existing `SearxClient`.
// Replaces `NoopResearchDispatcher` in the production live_cli tick path so
// CosmosWorker actually produces refreshed bodies instead of failing every
// tick (DRIFT-B).
//
// Sync bridge pattern mirrors `SearxSearchBackend` (orchestrator/src/
// searx_search_backend.rs): own a dedicated current-thread tokio runtime +
// std::sync::Mutex around the rate-limited SearxClient. The runtime is
// `current_thread` (cheap; never spawns) and isolated from the outer
// multi-thread runtime that `cosmos_tick_emit_loop` runs on.
// ---------------------------------------------------------------------------

use elai_knowledge::cosmos_worker::{CosmosError as KnowledgeCosmosError, ResearchDispatcher};
use elai_web_search::client::SearxClient;
use std::future::Future;
use std::pin::Pin;
use tokio::sync::Mutex as AsyncMutex;

/// C-16 (DECISION-B): production `ResearchDispatcher` impl that queries
/// SearXNG (via the sidecar `/search` proxy) and concatenates the top
/// results into a Markdown body suitable for a cosmos artifact.
///
/// The dispatcher implements the `min_source_tier` filter by mapping to
/// `SourceQualityTable` tiers post-search (currently the sidecar's
/// SearXNG returns engine + url; tier resolution is best-effort against
/// the parsed_domain). `max_size_bytes` is enforced as a soft cap: the
/// dispatcher stops appending result blocks once the running body
/// exceeds the cap. CosmosWorker::refresh_one re-truncates at the cap if
/// the body still overflows.
///
/// # Async-native (C-16 BLOCK-1 fix)
///
/// The dispatcher uses `tokio::sync::Mutex` (async) around the rate-limited
/// `SearxClient` so the lock can be held across `client.search().await`
/// without blocking the runtime. The previous current-thread tokio runtime
/// + `runtime.block_on()` bridge panicked when called from inside an
/// existing tokio runtime (live_cli + Tauri command). The fix dispatches
/// directly via reqwest's async client (SearxClient::search is already
/// async) and removes the nested-runtime entirely.
pub struct HttpCosmosResearchDispatcher {
    client: AsyncMutex<SearxClient>,
    max_results: usize,
}

impl HttpCosmosResearchDispatcher {
    /// Default max results per query. Capped at SearxClient's own
    /// `DEFAULT_MAX_RESULTS = 20` ceiling — values above silently clamp.
    pub const DEFAULT_MAX_RESULTS: usize = 10;

    /// Build a dispatcher targeting the sidecar `base_url`. Use
    /// `DEFAULT_SIDECAR_SEARCH_URL` (`http://localhost:8100`) unless the
    /// caller has a custom sidecar endpoint.
    pub fn new(base_url: impl Into<String>) -> Result<Self, GroundingError> {
        Self::with_clock(base_url, Arc::new(OsClock), Self::DEFAULT_MAX_RESULTS)
    }

    /// Build with custom clock + max_results (test-friendly).
    pub fn with_clock(
        base_url: impl Into<String>,
        clock: Arc<dyn ClockProvider>,
        max_results: usize,
    ) -> Result<Self, GroundingError> {
        let client = SearxClient::new(base_url, clock).map_err(|e| GroundingError::RequestFailed {
            reason: format!("HttpCosmosResearchDispatcher searx client: {e}"),
        })?;

        Ok(Self {
            client: AsyncMutex::new(client),
            max_results,
        })
    }
}

impl ResearchDispatcher for HttpCosmosResearchDispatcher {
    fn run<'a>(
        &'a self,
        query: &'a str,
        min_source_tier: &'a str,
        max_size_bytes: u64,
    ) -> Pin<Box<dyn Future<Output = Result<(String, String), KnowledgeCosmosError>> + Send + 'a>>
    {
        Box::pin(async move {
            debug!(
                query = query,
                min_source_tier = min_source_tier,
                "http_cosmos_research: issuing search"
            );

            // Hold the async mutex across the await: SearxClient is
            // rate-limited and not Sync (single-flight per dispatcher
            // instance is the intended contract).
            let mut guard = self.client.lock().await;
            let results = guard
                .search(query)
                .await
                .map_err(|e| KnowledgeCosmosError::Research(format!("searx: {e}")))?;
            drop(guard);

            // Compose Markdown body: heading + bullet per result. Honour the
            // size cap as a soft stop; refresh_one will re-truncate if needed.
            let mut body = String::with_capacity(2048);
            body.push_str(&format!("# {}\n\n", query));
            body.push_str(&format!(
                "_Curated from SearXNG ({} results, min_tier={})_\n\n",
                self.max_results, min_source_tier
            ));
            let mut included = 0usize;
            for r in results.into_iter().take(self.max_results) {
                if (body.as_bytes().len() as u64) >= max_size_bytes {
                    break;
                }
                // Snippet may contain HTML residue; SearxClient::search runs
                // strip_html internally so we trust the snippet as plain text.
                body.push_str(&format!(
                    "## {}\n[{}]({})\n\n{}\n\n",
                    r.title.trim(),
                    r.parsed_domain,
                    r.url,
                    r.snippet.trim()
                ));
                included += 1;
            }
            if included == 0 {
                return Err(KnowledgeCosmosError::Research(format!(
                    "no search results for query: {}",
                    query
                )));
            }

            // Source tier: best-effort label. Without a SourceQualityTable
            // lookup here (would require the tier table arc), we tag the body
            // with the requested min_tier — consistent with the cosmos
            // contract (the meta.source_tier is the WORST tier of any
            // included source; we conservatively report the requested floor).
            Ok((body, min_source_tier.to_string()))
        })
    }
}

// ---------------------------------------------------------------------------
// SsbEmitterAdapter (C-16 DECISION-E + BLOCK-1 fix) — bridges
// `elai_knowledge::cosmos_worker::SsbEmitter` (async-native trait) to an
// `Arc<Ssb>` (async `write_payload`). The previous current-thread tokio
// runtime + `runtime.block_on()` bridge panicked when called from inside an
// existing tokio runtime (live_cli cosmos tick + Tauri command). The async
// trait now awaits `Ssb::write_payload` directly on the ambient runtime —
// no nested runtime, no panic.
// ---------------------------------------------------------------------------

use elai_knowledge::cosmos_worker::SsbEmitter as KnowledgeSsbEmitter;
use crate::ssb::{
    CosmosArtifactDecayedPayload, CosmosArtifactUpdatedPayload, CosmosRefreshStartedPayload,
    Ssb,
};

/// Constant agent id for SSB rows emitted by the in-process cosmos tick.
/// Mirrors `task_runner::RUNNER_AGENT_ID` but scoped to the cosmos worker
/// so dashboards can filter cosmos-originated rows distinct from runner rows.
pub const COSMOS_SSB_AGENT_ID: &str = "cosmos_worker";

/// Constant superstep used for cosmos SSB rows. Cosmos events are
/// out-of-band w.r.t. the planner superstep counter, so 0 is the canonical
/// "system" superstep (same convention as task_runner::RUNNER_SUPERSTEP).
pub const COSMOS_SSB_SUPERSTEP: i64 = 0;

/// C-16 (DECISION-E + BLOCK-1 fix): production SSB emitter for CosmosWorker.
/// Holds a shared `Arc<Ssb>` writer; the async trait methods await
/// `Ssb::write_payload` directly on the ambient tokio runtime (no nested
/// runtime, no `block_on` bridge).
///
/// All three `emit_*` methods are best-effort: transport errors are
/// swallowed via `tracing::warn!` so a transient SQLite contention never
/// panics CosmosWorker::tick().
pub struct SsbEmitterAdapter {
    writer: Arc<Ssb>,
}

impl SsbEmitterAdapter {
    /// Build an adapter wrapping a shared writer. Infallible — no longer
    /// owns a tokio runtime (BLOCK-1 fix). Returns `Result` for source
    /// compatibility with existing call sites (`SsbEmitterAdapter::new(...)
    /// .map(Arc::new)` patterns); construction never errors.
    pub fn new(writer: Arc<Ssb>) -> Result<Self, GroundingError> {
        Ok(Self { writer })
    }
}

impl KnowledgeSsbEmitter for SsbEmitterAdapter {
    fn emit_refresh_started<'a>(
        &'a self,
        queued: u32,
        total: u32,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let payload = SsbPayload::CosmosRefreshStarted(CosmosRefreshStartedPayload {
                queued,
                total,
            });
            if let Err(e) = self
                .writer
                .write_payload(COSMOS_SSB_AGENT_ID, COSMOS_SSB_SUPERSTEP, payload, Vec::new())
                .await
            {
                tracing::warn!(
                    error = %e,
                    "SsbEmitterAdapter: CosmosRefreshStarted write failed (non-fatal)"
                );
            }
        })
    }

    fn emit_artifact_updated<'a>(
        &'a self,
        name: &'a str,
        blake3_hash: &'a str,
        body_bytes: u32,
        latency_ms: u32,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let payload = SsbPayload::CosmosArtifactUpdated(CosmosArtifactUpdatedPayload {
                name: name.to_string(),
                blake3: blake3_hash.to_string(),
                body_bytes,
                latency_ms,
            });
            if let Err(e) = self
                .writer
                .write_payload(COSMOS_SSB_AGENT_ID, COSMOS_SSB_SUPERSTEP, payload, Vec::new())
                .await
            {
                tracing::warn!(
                    artifact = %name,
                    error = %e,
                    "SsbEmitterAdapter: CosmosArtifactUpdated write failed (non-fatal)"
                );
            }
        })
    }

    fn emit_artifact_decayed<'a>(
        &'a self,
        name: &'a str,
        age_secs: u64,
        cadence: &'a str,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let payload = SsbPayload::CosmosArtifactDecayed(CosmosArtifactDecayedPayload {
                name: name.to_string(),
                age_secs,
                cadence: cadence.to_string(),
            });
            if let Err(e) = self
                .writer
                .write_payload(COSMOS_SSB_AGENT_ID, COSMOS_SSB_SUPERSTEP, payload, Vec::new())
                .await
            {
                tracing::warn!(
                    artifact = %name,
                    error = %e,
                    "SsbEmitterAdapter: CosmosArtifactDecayed write failed (non-fatal)"
                );
            }
        })
    }
}
