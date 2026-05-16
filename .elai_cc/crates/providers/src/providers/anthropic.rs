use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, UNIX_EPOCH};

use elai_core::config::OAuthConfig;
use elai_core::format_usd;
use crate::oauth::{load_oauth_credentials, save_oauth_credentials, OAuthRefreshRequest, OAuthTokenExchangeRequest};
use crate::telemetry_types::{AnalyticsEvent, AnthropicRequestProfile, ClientIdentity, SessionTracer};
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::error::ApiError;
use crate::prompt_cache::{PromptCache, PromptCacheRecord, PromptCacheStats};

use super::{preflight_message_request, Provider, ProviderFuture};
use crate::sse::SseParser;
use crate::types::{
    CacheControl, MessageDeltaEvent, MessageRequest, MessageResponse, StreamEvent, SystemBlock,
    Usage,
};

pub const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const REQUEST_ID_HEADER: &str = "request-id";
const ALT_REQUEST_ID_HEADER: &str = "x-request-id";
const DEFAULT_INITIAL_BACKOFF: Duration = Duration::from_secs(1);
const DEFAULT_MAX_BACKOFF: Duration = Duration::from_secs(60);
const DEFAULT_MAX_RETRIES: u32 = 4;

// Claude Code identification headers. Required for OAuth-bearer requests to
// avoid Anthropic's stricter default rate-limit bucket. Source: leaked
// claude-code-main src/utils/http.ts + the dashboard's start_chat_stream
// at dashboard-v2/src-tauri/src/commands.rs:2300-2308 and :3061-3077.
// Without these, OAuth tokens silently route to a near-zero quota and
// every /v1/messages returns 429 even on Max-20x plans.
// Bumped to current Claude Code CLI version per anthropic-rate-limits-findings.md
// PART 4 / PART 5. Stale UA versions are a documented Anthropic abuse-detection
// fingerprint surface. Verified 2026-05-15 against @anthropic-ai/claude-code npm.
// Re-bump cadence: ~quarterly per Claude Code release.
const CLAUDE_CLI_USER_AGENT: &str = "claude-cli/2.1.143 (external, cli)";
const CLAUDE_X_APP_HEADER: &str = "cli";
const OAUTH_BETA_HEADER: &str = "oauth-2025-04-20";
const OAUTH_ANTHROPIC_VERSION: &str = "2023-06-01";

/// Mandatory system-prompt prefix for Opus + Sonnet OAuth requests. Source:
/// anthropic-rate-limits-findings.md PART 3 / CONDITION 3 — Anthropic's
/// /v1/messages OAuth path validates that the `system` field starts with this
/// EXACT literal for Opus and Sonnet models. Missing it returns
/// `400 invalid_request_error: Error`. Haiku is exempt.
///
/// Implementation: `prepend_oauth_system_prefix` injects this as the first
/// `SystemBlock` (idempotent — leaves an existing prefix block alone) when
/// auth is BearerToken AND model is opus/sonnet. The block is wrapped in
/// ephemeral cache_control to amortise its token cost across the rolling
/// 5-minute cache window (well under Anthropic's 4-marker per-request cap).
const CLAUDE_CODE_OAUTH_SYSTEM_PREFIX: &str =
    "You are Claude Code, Anthropic's official CLI for Claude.";

/// Stable per-process UUIDv4-shaped session id for `X-Claude-Code-Session-Id`.
/// Mirrors dashboard_cli_session_id at dashboard-v2/src-tauri/src/commands.rs:2308.
/// Anthropic's log pipeline may parse this; non-conforming shapes route to a
/// degraded analytics path with stricter throttles. Uses getrandom (already a
/// providers crate dep) for the 16 random bytes; output has RFC 4122 v4
/// version/variant bits set.
fn claude_code_session_id() -> &'static str {
    use std::sync::OnceLock;
    static ID: OnceLock<String> = OnceLock::new();
    ID.get_or_init(|| {
        let mut bytes = [0u8; 16];
        // Fallback to a pid+timestamp-derived seed if getrandom fails (it
        // shouldn't on supported platforms). Non-cryptographic; only needs
        // to look like UUIDv4 to Anthropic's parser.
        if getrandom::fill(&mut bytes).is_err() {
            let pid = std::process::id().to_le_bytes();
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
                .to_le_bytes();
            bytes[..4].copy_from_slice(&pid);
            bytes[4..].copy_from_slice(&ts[..12]);
        }
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5],
            bytes[6], bytes[7],
            bytes[8], bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        )
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthSource {
    None,
    ApiKey(String),
    BearerToken(String),
    ApiKeyAndBearer {
        api_key: String,
        bearer_token: String,
    },
}

impl AuthSource {
    pub fn from_env() -> Result<Self, ApiError> {
        let api_key = read_env_non_empty("ANTHROPIC_API_KEY")?;
        let auth_token = read_env_non_empty("ANTHROPIC_AUTH_TOKEN")?;
        match (api_key, auth_token) {
            (Some(api_key), Some(bearer_token)) => Ok(Self::ApiKeyAndBearer {
                api_key,
                bearer_token,
            }),
            (Some(api_key), None) => Ok(Self::ApiKey(api_key)),
            (None, Some(bearer_token)) => Ok(Self::BearerToken(bearer_token)),
            (None, None) => Err(ApiError::missing_credentials(
                "Anthropic",
                &["ANTHROPIC_AUTH_TOKEN", "ANTHROPIC_API_KEY"],
            )),
        }
    }

    #[must_use]
    pub fn api_key(&self) -> Option<&str> {
        match self {
            Self::ApiKey(api_key) | Self::ApiKeyAndBearer { api_key, .. } => Some(api_key),
            Self::None | Self::BearerToken(_) => None,
        }
    }

    #[must_use]
    pub fn bearer_token(&self) -> Option<&str> {
        match self {
            Self::BearerToken(token)
            | Self::ApiKeyAndBearer {
                bearer_token: token,
                ..
            } => Some(token),
            Self::None | Self::ApiKey(_) => None,
        }
    }

    #[must_use]
    pub fn masked_authorization_header(&self) -> &'static str {
        if self.bearer_token().is_some() {
            "Bearer [REDACTED]"
        } else {
            "<absent>"
        }
    }

    pub fn apply(&self, mut request_builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(api_key) = self.api_key() {
            request_builder = request_builder.header("x-api-key", api_key);
        }
        if let Some(token) = self.bearer_token() {
            request_builder = request_builder.bearer_auth(token);
        }
        request_builder
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OAuthTokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
    #[serde(default)]
    pub scopes: Vec<String>,
}

impl From<OAuthTokenSet> for AuthSource {
    fn from(value: OAuthTokenSet) -> Self {
        Self::BearerToken(value.access_token)
    }
}

/// Response shape of GET `/api/oauth/usage`. Each field is the user's
/// rolling-window utilisation as a percentage (0-100). When either reads
/// 100 the corresponding subscription bucket is exhausted.
///
/// Fields documented in anthropic-rate-limits-findings.md PART 2.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct OAuthUsage {
    pub five_hour: f32,
    pub seven_day: f32,
}

impl OAuthUsage {
    /// True when either bucket is at or above the safe-burst threshold.
    /// Default threshold 80% (matches the findings doc's recommendation).
    #[must_use]
    pub fn should_throttle(&self, threshold_pct: f32) -> bool {
        self.five_hour >= threshold_pct || self.seven_day >= threshold_pct
    }
}

#[derive(Debug, Clone)]
pub struct AnthropicClient {
    http: reqwest::Client,
    auth: AuthSource,
    base_url: String,
    max_retries: u32,
    initial_backoff: Duration,
    max_backoff: Duration,
    request_profile: AnthropicRequestProfile,
    session_tracer: Option<SessionTracer>,
    prompt_cache: Option<PromptCache>,
    last_prompt_cache_record: Arc<Mutex<Option<PromptCacheRecord>>>,
}

impl AnthropicClient {
    #[must_use]
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            auth: AuthSource::ApiKey(api_key.into()),
            base_url: DEFAULT_BASE_URL.to_string(),
            max_retries: DEFAULT_MAX_RETRIES,
            initial_backoff: DEFAULT_INITIAL_BACKOFF,
            max_backoff: DEFAULT_MAX_BACKOFF,
            request_profile: AnthropicRequestProfile::default(),
            session_tracer: None,
            prompt_cache: None,
            last_prompt_cache_record: Arc::new(Mutex::new(None)),
        }
    }

    #[must_use]
    pub fn from_auth(auth: AuthSource) -> Self {
        Self {
            http: reqwest::Client::new(),
            auth,
            base_url: DEFAULT_BASE_URL.to_string(),
            max_retries: DEFAULT_MAX_RETRIES,
            initial_backoff: DEFAULT_INITIAL_BACKOFF,
            max_backoff: DEFAULT_MAX_BACKOFF,
            request_profile: AnthropicRequestProfile::default(),
            session_tracer: None,
            prompt_cache: None,
            last_prompt_cache_record: Arc::new(Mutex::new(None)),
        }
    }

    pub fn from_env() -> Result<Self, ApiError> {
        Ok(Self::from_auth(AuthSource::from_env_or_saved()?).with_base_url(read_base_url()))
    }

    #[must_use]
    pub fn with_auth_source(mut self, auth: AuthSource) -> Self {
        self.auth = auth;
        self
    }

    #[must_use]
    pub fn with_auth_token(mut self, auth_token: Option<String>) -> Self {
        match (
            self.auth.api_key().map(ToOwned::to_owned),
            auth_token.filter(|token| !token.is_empty()),
        ) {
            (Some(api_key), Some(bearer_token)) => {
                self.auth = AuthSource::ApiKeyAndBearer {
                    api_key,
                    bearer_token,
                };
            }
            (Some(api_key), None) => {
                self.auth = AuthSource::ApiKey(api_key);
            }
            (None, Some(bearer_token)) => {
                self.auth = AuthSource::BearerToken(bearer_token);
            }
            (None, None) => {
                self.auth = AuthSource::None;
            }
        }
        self
    }

    #[must_use]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    #[must_use]
    pub fn with_retry_policy(
        mut self,
        max_retries: u32,
        initial_backoff: Duration,
        max_backoff: Duration,
    ) -> Self {
        self.max_retries = max_retries;
        self.initial_backoff = initial_backoff;
        self.max_backoff = max_backoff;
        self
    }

    #[must_use]
    pub fn with_session_tracer(mut self, session_tracer: SessionTracer) -> Self {
        self.session_tracer = Some(session_tracer);
        self
    }

    #[must_use]
    pub fn with_client_identity(mut self, client_identity: ClientIdentity) -> Self {
        self.request_profile.client_identity = client_identity;
        self
    }

    #[must_use]
    pub fn with_beta(mut self, beta: impl Into<String>) -> Self {
        self.request_profile = self.request_profile.with_beta(beta);
        self
    }

    #[must_use]
    pub fn with_extra_body_param(mut self, key: impl Into<String>, value: Value) -> Self {
        self.request_profile = self.request_profile.with_extra_body(key, value);
        self
    }

    #[must_use]
    pub fn with_prompt_cache(mut self, prompt_cache: PromptCache) -> Self {
        self.prompt_cache = Some(prompt_cache);
        self
    }

    #[must_use]
    pub fn prompt_cache_stats(&self) -> Option<PromptCacheStats> {
        self.prompt_cache.as_ref().map(PromptCache::stats)
    }

    #[must_use]
    pub fn request_profile(&self) -> &AnthropicRequestProfile {
        &self.request_profile
    }

    #[must_use]
    pub fn session_tracer(&self) -> Option<&SessionTracer> {
        self.session_tracer.as_ref()
    }

    #[must_use]
    pub fn prompt_cache(&self) -> Option<&PromptCache> {
        self.prompt_cache.as_ref()
    }

    #[must_use]
    pub fn take_last_prompt_cache_record(&self) -> Option<PromptCacheRecord> {
        self.last_prompt_cache_record
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }

    #[must_use]
    pub fn with_request_profile(mut self, request_profile: AnthropicRequestProfile) -> Self {
        self.request_profile = request_profile;
        self
    }

    #[must_use]
    pub fn auth_source(&self) -> &AuthSource {
        &self.auth
    }

    pub async fn send_message(
        &self,
        request: &MessageRequest,
    ) -> Result<MessageResponse, ApiError> {
        let mut request = MessageRequest {
            stream: false,
            ..request.clone()
        };
        apply_anthropic_cache_markers(&mut request);

        if let Some(prompt_cache) = &self.prompt_cache {
            if let Some(response) = prompt_cache.lookup_completion(&request) {
                return Ok(response);
            }
        }

        preflight_message_request(&request)?;

        let response = self.send_with_retry(&request).await?;
        let request_id = request_id_from_headers(response.headers());
        let mut response = response
            .json::<MessageResponse>()
            .await
            .map_err(ApiError::from)?;
        if response.request_id.is_none() {
            response.request_id = request_id;
        }

        if let Some(prompt_cache) = &self.prompt_cache {
            let record = prompt_cache.record_response(&request, &response);
            self.store_last_prompt_cache_record(record);
        }
        if let Some(session_tracer) = &self.session_tracer {
            session_tracer.record_analytics(
                AnalyticsEvent::new("api", "message_usage")
                    .with_property(
                        "request_id",
                        response
                            .request_id
                            .clone()
                            .map_or(Value::Null, Value::String),
                    )
                    .with_property("total_tokens", Value::from(response.total_tokens()))
                    .with_property(
                        "estimated_cost_usd",
                        Value::String(format_usd(
                            response
                                .usage
                                .estimated_cost_usd(&response.model)
                                .total_cost_usd(),
                        )),
                    ),
            );
        }
        Ok(response)
    }

    pub async fn stream_message(
        &self,
        request: &MessageRequest,
    ) -> Result<MessageStream, ApiError> {
        let mut prepared = request.clone();
        apply_anthropic_cache_markers(&mut prepared);
        preflight_message_request(&prepared)?;
        let response = self
            .send_with_retry(&prepared.clone().with_streaming())
            .await?;
        Ok(MessageStream {
            request_id: request_id_from_headers(response.headers()),
            response,
            parser: SseParser::new(),
            pending: VecDeque::new(),
            done: false,
            request: prepared,
            prompt_cache: self.prompt_cache.clone(),
            latest_usage: None,
            usage_recorded: false,
            last_prompt_cache_record: Arc::clone(&self.last_prompt_cache_record),
        })
    }

    pub async fn exchange_oauth_code(
        &self,
        config: &OAuthConfig,
        request: &OAuthTokenExchangeRequest,
    ) -> Result<OAuthTokenSet, ApiError> {
        // Claude Code's token endpoint expects JSON, NOT form-urlencoded.
        // Source of truth: leaked claude-code-main src/services/oauth/client.ts
        // exchangeCodeForTokens() → axios.post(TOKEN_URL, requestBody,
        // { headers: { 'Content-Type': 'application/json' }, timeout: 15000 }).
        let response = self
            .http
            .post(&config.token_url)
            .json(&request.form_params())
            .send()
            .await
            .map_err(ApiError::from)?;
        let response = expect_success(response).await?;
        response
            .json::<OAuthTokenSet>()
            .await
            .map_err(ApiError::from)
    }

    pub async fn refresh_oauth_token(
        &self,
        config: &OAuthConfig,
        request: &OAuthRefreshRequest,
    ) -> Result<OAuthTokenSet, ApiError> {
        // Same JSON contract as exchange_oauth_code — see comment above.
        let response = self
            .http
            .post(&config.token_url)
            .json(&request.form_params())
            .send()
            .await
            .map_err(ApiError::from)?;
        let response = expect_success(response).await?;
        response
            .json::<OAuthTokenSet>()
            .await
            .map_err(ApiError::from)
    }

    /// Pre-flight quota check against Anthropic's reverse-engineered
    /// `/api/oauth/usage` endpoint. Returns the user's `five_hour` and
    /// `seven_day` rolling-bucket utilisation as percentages 0-100.
    ///
    /// Source: anthropic-rate-limits-findings.md PART 2 — endpoint is
    /// undocumented in T1 but confirmed via reverse engineering. Only
    /// works with subscription-client OAuth tokens (Console-client tokens
    /// will 401 here, which is itself a useful diagnostic).
    ///
    /// Use this BEFORE bursting a workload. If `five_hour > 80` or
    /// `seven_day > 80`, throttle aggressively or stop — burning quota
    /// from this tool also locks out the user's interactive Claude Code
    /// session (claude.ai web + CLI + VS Code extension share the same
    /// OAuth identity bucket).
    ///
    /// Requires `self.auth` to be `AuthSource::BearerToken(_)`. Returns
    /// `ApiError::Auth` otherwise.
    pub async fn fetch_oauth_usage(&self) -> Result<OAuthUsage, ApiError> {
        if !matches!(&self.auth, AuthSource::BearerToken(_)) {
            return Err(ApiError::Auth(
                "fetch_oauth_usage requires AuthSource::BearerToken".to_string(),
            ));
        }
        let request_url = format!(
            "{}/api/oauth/usage",
            self.base_url.trim_end_matches('/')
        );
        let mut builder = self.http.get(&request_url);
        builder = self.auth.apply(builder);
        builder = builder
            .header("anthropic-beta", OAUTH_BETA_HEADER)
            .header("anthropic-version", OAUTH_ANTHROPIC_VERSION)
            .header("user-agent", CLAUDE_CLI_USER_AGENT)
            .header("x-app", CLAUDE_X_APP_HEADER)
            .header("x-claude-code-session-id", claude_code_session_id());
        let response = builder.send().await.map_err(ApiError::from)?;
        let response = expect_success(response).await?;
        response.json::<OAuthUsage>().await.map_err(ApiError::from)
    }

    async fn send_with_retry(
        &self,
        request: &MessageRequest,
    ) -> Result<reqwest::Response, ApiError> {
        let mut attempts = 0;
        // Always assigned before the loop reads it (via match arms below); the
        // init exists so the post-loop `last_error.expect(..)` is well-defined
        // even if the compiler can't prove all paths through the loop.
        #[allow(unused_assignments)]
        let mut last_error: Option<ApiError> = None;

        loop {
            attempts += 1;
            if let Some(session_tracer) = &self.session_tracer {
                session_tracer.record_http_request_started(
                    attempts,
                    "POST",
                    "/v1/messages",
                    Map::new(),
                );
            }
            match self.send_raw_request(request).await {
                Ok(response) => match expect_success(response).await {
                    Ok(response) => {
                        if let Some(session_tracer) = &self.session_tracer {
                            session_tracer.record_http_request_succeeded(
                                attempts,
                                "POST",
                                "/v1/messages",
                                response.status().as_u16(),
                                request_id_from_headers(response.headers()),
                                Map::new(),
                            );
                        }
                        return Ok(response);
                    }
                    Err(error) if error.is_retryable() && attempts <= self.max_retries + 1 => {
                        self.record_request_failure(attempts, &error);
                        last_error = Some(error);
                    }
                    Err(error) => {
                        self.record_request_failure(attempts, &error);
                        return Err(error);
                    }
                },
                Err(error) if error.is_retryable() && attempts <= self.max_retries + 1 => {
                    self.record_request_failure(attempts, &error);
                    last_error = Some(error);
                }
                Err(error) => {
                    self.record_request_failure(attempts, &error);
                    return Err(error);
                }
            }

            if attempts > self.max_retries {
                break;
            }

            // Honor server-provided Retry-After header on 429/503 in preference
            // to exponential backoff — cuts 429 cascade duration in cloud_mode.
            let sleep_duration = match &last_error {
                Some(ApiError::Api { retry_after: Some(d), .. }) => *d,
                _ => self.backoff_for_attempt(attempts)?,
            };
            tokio::time::sleep(sleep_duration).await;
        }

        Err(ApiError::RetriesExhausted {
            attempts,
            last_error: Box::new(last_error.expect("retry loop must capture an error")),
        })
    }

    async fn send_raw_request(
        &self,
        request: &MessageRequest,
    ) -> Result<reqwest::Response, ApiError> {
        let request_url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        let request_builder = self
            .http
            .post(&request_url)
            .header("content-type", "application/json");
        let mut request_builder = self.auth.apply(request_builder);

        // OAuth bearer requests must look like Claude Code to Anthropic's
        // rate-limit-bucket router. The dashboard's start_chat_stream
        // (commands.rs:3061-3077) sends exactly this set; without it, OAuth
        // tokens route to a near-zero quota and every call returns a
        // generic 429 rate_limit_error even on Max-20x plans.
        let is_oauth_bearer = matches!(&self.auth, AuthSource::BearerToken(_));
        if is_oauth_bearer {
            request_builder = request_builder
                .header("anthropic-version", OAUTH_ANTHROPIC_VERSION)
                .header("anthropic-beta", OAUTH_BETA_HEADER)
                .header("user-agent", CLAUDE_CLI_USER_AGENT)
                .header("x-app", CLAUDE_X_APP_HEADER)
                .header("x-claude-code-session-id", claude_code_session_id());
        } else {
            for (header_name, header_value) in self.request_profile.header_pairs() {
                request_builder = request_builder.header(header_name, header_value);
            }
        }

        // OAuth requests against Opus/Sonnet REQUIRE the Claude Code system
        // prefix as the first SystemBlock; Anthropic returns 400 without it.
        // We mutate a local clone so callers' MessageRequest is untouched.
        let body_request = if is_oauth_bearer {
            let mut owned = request.clone();
            prepend_oauth_system_prefix(&mut owned);
            std::borrow::Cow::Owned(owned)
        } else {
            std::borrow::Cow::Borrowed(request)
        };
        let request_body = self.request_profile.render_json_body(body_request.as_ref())?;
        request_builder = request_builder.json(&request_body);
        request_builder.send().await.map_err(ApiError::from)
    }

    fn record_request_failure(&self, attempt: u32, error: &ApiError) {
        if let Some(session_tracer) = &self.session_tracer {
            session_tracer.record_http_request_failed(
                attempt,
                "POST",
                "/v1/messages",
                error.to_string(),
                error.is_retryable(),
                Map::new(),
            );
        }
    }

    fn store_last_prompt_cache_record(&self, record: PromptCacheRecord) {
        *self
            .last_prompt_cache_record
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(record);
    }

    fn backoff_for_attempt(&self, attempt: u32) -> Result<Duration, ApiError> {
        let Some(multiplier) = 1_u32.checked_shl(attempt.saturating_sub(1)) else {
            return Err(ApiError::BackoffOverflow {
                attempt,
                base_delay: self.initial_backoff,
            });
        };
        Ok(self
            .initial_backoff
            .checked_mul(multiplier)
            .map_or(self.max_backoff, |delay| delay.min(self.max_backoff)))
    }
}

impl AuthSource {
    pub fn from_env_or_saved() -> Result<Self, ApiError> {
        if let Some(api_key) = read_env_non_empty("ANTHROPIC_API_KEY")? {
            return match read_env_non_empty("ANTHROPIC_AUTH_TOKEN")? {
                Some(bearer_token) => Ok(Self::ApiKeyAndBearer {
                    api_key,
                    bearer_token,
                }),
                None => Ok(Self::ApiKey(api_key)),
            };
        }
        if let Some(bearer_token) = read_env_non_empty("ANTHROPIC_AUTH_TOKEN")? {
            return Ok(Self::BearerToken(bearer_token));
        }
        match load_saved_oauth_token() {
            Ok(Some(token_set)) if oauth_token_is_expired(&token_set) => {
                if token_set.refresh_token.is_some() {
                    Err(ApiError::Auth(
                        "saved OAuth token is expired; load runtime OAuth config to refresh it"
                            .to_string(),
                    ))
                } else {
                    Err(ApiError::ExpiredOAuthToken)
                }
            }
            Ok(Some(token_set)) => Ok(Self::BearerToken(token_set.access_token)),
            Ok(None) => Err(ApiError::missing_credentials(
                "Anthropic",
                &["ANTHROPIC_AUTH_TOKEN", "ANTHROPIC_API_KEY"],
            )),
            Err(error) => Err(error),
        }
    }
}

/// Returns true when the saved OAuth access token must be refreshed before
/// use. Three cases:
///
/// 1. `expires_at = Some(t)` AND `t <= now`: the recorded TTL has elapsed.
/// 2. `expires_at = None` AND `refresh_token = Some(_)`: the saved record
///    omits the TTL (some Claude Code CLI versions persist `expiresAt: null`
///    even though the access token still has a finite server-side TTL).
///    Force-refresh on next use so the live request never 401s on a token
///    the local code thought was perpetual. anthropic-rate-limits-findings.md
///    PART 1 documents this short-circuit as a recurring bug-class.
/// 3. `expires_at = None` AND `refresh_token = None`: nothing we can do —
///    the caller will see the live 401 and surface it. Returns false to
///    preserve prior behaviour for non-refreshable tokens.
#[must_use]
pub fn oauth_token_is_expired(token_set: &OAuthTokenSet) -> bool {
    match token_set.expires_at {
        Some(expires_at) => expires_at <= now_unix_timestamp(&elai_core::OsClock),
        None => token_set.refresh_token.is_some(),
    }
}

pub fn resolve_saved_oauth_token(config: &OAuthConfig) -> Result<Option<OAuthTokenSet>, ApiError> {
    let Some(token_set) = load_saved_oauth_token()? else {
        return Ok(None);
    };
    resolve_saved_oauth_token_set(config, token_set).map(Some)
}

pub fn has_auth_from_env_or_saved() -> Result<bool, ApiError> {
    Ok(read_env_non_empty("ANTHROPIC_API_KEY")?.is_some()
        || read_env_non_empty("ANTHROPIC_AUTH_TOKEN")?.is_some()
        || load_saved_oauth_token()?.is_some())
}

/// Canonical Claude Code OAuth config used by the dashboard and CLI login
/// flows. Hardcoded here so the providers crate is self-sufficient — any
/// caller can call `resolve_startup_auth_source` (or
/// `resolve_auth_with_auto_refresh`) without pulling cli/dashboard crates.
///
/// Source of truth (mirrored from cli/src/auth.rs::default_oauth_config):
/// leaked claude-code-main src/constants/oauth.ts PROD_OAUTH_CONFIG. The
/// scopes are the Claude.ai-subscriber union (Pro/Max plans authenticate
/// without consuming API credits).
#[must_use]
pub fn default_claude_code_oauth_config() -> OAuthConfig {
    OAuthConfig {
        client_id: "9d1c250a-e61b-44d9-88ed-5944d1962f5e".to_string(),
        authorize_url: "https://claude.com/cai/oauth/authorize".to_string(),
        token_url: "https://platform.claude.com/v1/oauth/token".to_string(),
        callback_port: None,
        manual_redirect_url: Some(
            "https://platform.claude.com/oauth/code/callback".to_string(),
        ),
        scopes: vec![
            "org:create_api_key".to_string(),
            "user:profile".to_string(),
            "user:inference".to_string(),
            "user:sessions:claude_code".to_string(),
            "user:mcp_servers".to_string(),
            "user:file_upload".to_string(),
        ],
    }
}

/// One-shot auto-refresh helper for callers that don't want to thread the
/// OAuthConfig through their own loader. Wraps `resolve_startup_auth_source`
/// with `default_claude_code_oauth_config` as the loader. On successful
/// refresh the new token is persisted to ELAI_CONFIG_HOME/credentials.json
/// (handled inside `resolve_saved_oauth_token_set`).
///
/// Returns the AuthSource ready to construct an AnthropicClient. Order:
/// 1. `ANTHROPIC_API_KEY` env → ApiKey
/// 2. `ANTHROPIC_AUTH_TOKEN` env → BearerToken
/// 3. Saved OAuth + not expired → BearerToken with saved access_token
/// 4. Saved OAuth + expired + refresh present → refresh via Claude Code
///    OAuth config, save new token, return BearerToken with new access_token
/// 5. Otherwise → `MissingCredentials` error
pub fn resolve_auth_with_auto_refresh() -> Result<AuthSource, ApiError> {
    resolve_startup_auth_source(|| Ok(Some(default_claude_code_oauth_config())))
}

pub fn resolve_startup_auth_source<F>(load_oauth_config: F) -> Result<AuthSource, ApiError>
where
    F: FnOnce() -> Result<Option<OAuthConfig>, ApiError>,
{
    if let Some(api_key) = read_env_non_empty("ANTHROPIC_API_KEY")? {
        return match read_env_non_empty("ANTHROPIC_AUTH_TOKEN")? {
            Some(bearer_token) => Ok(AuthSource::ApiKeyAndBearer {
                api_key,
                bearer_token,
            }),
            None => Ok(AuthSource::ApiKey(api_key)),
        };
    }
    if let Some(bearer_token) = read_env_non_empty("ANTHROPIC_AUTH_TOKEN")? {
        return Ok(AuthSource::BearerToken(bearer_token));
    }

    let Some(token_set) = load_saved_oauth_token()? else {
        return Err(ApiError::missing_credentials(
            "Anthropic",
            &["ANTHROPIC_AUTH_TOKEN", "ANTHROPIC_API_KEY"],
        ));
    };
    if !oauth_token_is_expired(&token_set) {
        return Ok(AuthSource::BearerToken(token_set.access_token));
    }
    if token_set.refresh_token.is_none() {
        return Err(ApiError::ExpiredOAuthToken);
    }

    let Some(config) = load_oauth_config()? else {
        return Err(ApiError::Auth(
            "saved OAuth token is expired; runtime OAuth config is missing".to_string(),
        ));
    };
    Ok(AuthSource::from(resolve_saved_oauth_token_set(
        &config, token_set,
    )?))
}

fn resolve_saved_oauth_token_set(
    config: &OAuthConfig,
    token_set: OAuthTokenSet,
) -> Result<OAuthTokenSet, ApiError> {
    if !oauth_token_is_expired(&token_set) {
        return Ok(token_set);
    }
    let Some(refresh_token) = token_set.refresh_token.clone() else {
        return Err(ApiError::ExpiredOAuthToken);
    };
    let client = AnthropicClient::from_auth(AuthSource::None).with_base_url(read_base_url());
    let refreshed = client_runtime_block_on(async {
        client
            .refresh_oauth_token(
                config,
                &OAuthRefreshRequest::from_config(
                    config,
                    refresh_token,
                    Some(token_set.scopes.clone()),
                ),
            )
            .await
    })?;
    let resolved = OAuthTokenSet {
        access_token: refreshed.access_token,
        refresh_token: refreshed.refresh_token.or(token_set.refresh_token),
        expires_at: refreshed.expires_at,
        scopes: refreshed.scopes,
    };
    save_oauth_credentials(&crate::oauth::OAuthTokenSet {
        access_token: resolved.access_token.clone(),
        refresh_token: resolved.refresh_token.clone(),
        expires_at: resolved.expires_at,
        scopes: resolved.scopes.clone(),
    })
    .map_err(ApiError::from)?;
    Ok(resolved)
}

fn client_runtime_block_on<F, T>(future: F) -> Result<T, ApiError>
where
    F: std::future::Future<Output = Result<T, ApiError>>,
{
    tokio::runtime::Runtime::new()
        .map_err(ApiError::from)?
        .block_on(future)
}

fn load_saved_oauth_token() -> Result<Option<OAuthTokenSet>, ApiError> {
    let token_set = load_oauth_credentials().map_err(ApiError::from)?;
    Ok(token_set.map(|token_set| OAuthTokenSet {
        access_token: token_set.access_token,
        refresh_token: token_set.refresh_token,
        expires_at: token_set.expires_at,
        scopes: token_set.scopes,
    }))
}

fn now_unix_timestamp(clock: &dyn elai_core::ClockProvider) -> u64 {
    clock
        .system_now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

fn read_env_non_empty(key: &str) -> Result<Option<String>, ApiError> {
    match std::env::var(key) {
        Ok(value) if !value.is_empty() => Ok(Some(value)),
        Ok(_) | Err(std::env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(ApiError::from(error)),
    }
}

#[cfg(test)]
fn read_api_key() -> Result<String, ApiError> {
    let auth = AuthSource::from_env_or_saved()?;
    auth.api_key()
        .or_else(|| auth.bearer_token())
        .map(ToOwned::to_owned)
        .ok_or(ApiError::missing_credentials(
            "Anthropic",
            &["ANTHROPIC_AUTH_TOKEN", "ANTHROPIC_API_KEY"],
        ))
}

#[cfg(test)]
fn read_auth_token() -> Option<String> {
    read_env_non_empty("ANTHROPIC_AUTH_TOKEN")
        .ok()
        .and_then(std::convert::identity)
}

#[must_use]
pub fn read_base_url() -> String {
    std::env::var("ANTHROPIC_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
}

/// Apply Anthropic ephemeral `cache_control` markers to the stable prefix
/// blocks of a `MessageRequest` in place.
///
/// Strategy (well under Anthropic's 4-marker per-request limit):
/// - mark the LAST `SystemBlock` if any system blocks are present
/// - mark the LAST `ToolDefinition` if any tools are present
///
/// Already-marked blocks are left alone; this fn is idempotent. Markers
/// are an Anthropic-only feature and are silently dropped on the
/// OpenAI-compat path (see `openai_compat::build_chat_completion_request`).
///
/// B-27 grounding: prior to this change the `PromptCache` module tracked
/// `cache_creation_input_tokens`/`cache_read_input_tokens` from response
/// usage, but no request ever carried a marker, so the cache hit rate
/// was always 0 and the spend tracker reported full input cost on every
/// call. Wiring markers makes the cloud-mode spend ledger honest.
fn apply_anthropic_cache_markers(request: &mut MessageRequest) {
    if let Some(blocks) = request.system.as_mut() {
        if let Some(last) = blocks.last_mut() {
            if last.cache_control.is_none() {
                last.cache_control = Some(CacheControl::ephemeral());
            }
        }
    }
    if let Some(tools) = request.tools.as_mut() {
        if let Some(last) = tools.last_mut() {
            if last.cache_control.is_none() {
                last.cache_control = Some(CacheControl::ephemeral());
            }
        }
    }
}

/// Returns true when `model` is an Opus or Sonnet model that requires the
/// Claude Code OAuth system-prompt prefix. Haiku is exempt per
/// anthropic-rate-limits-findings.md PART 3 / CONDITION 3.
fn model_requires_oauth_system_prefix(model: &str) -> bool {
    let lower = model.to_ascii_lowercase();
    lower.starts_with("claude-opus") || lower.starts_with("claude-sonnet")
}

/// Inject the mandatory `CLAUDE_CODE_OAUTH_SYSTEM_PREFIX` as the first
/// `SystemBlock` of `request` when (a) the model is Opus or Sonnet AND
/// (b) the existing first system block (if any) does not already start
/// with the prefix. The injected block is wrapped in ephemeral
/// `cache_control` so its token cost amortises across the 5-minute cache
/// window (within Anthropic's 4-marker per-request cap — see
/// `apply_anthropic_cache_markers` above).
///
/// Idempotent: calling twice on the same request does not double-prepend.
/// Safe on Haiku and other non-Opus/Sonnet models (no-op).
///
/// Anthropic's /v1/messages OAuth path returns
/// `400 invalid_request_error: Error` when this prefix is absent on
/// Opus/Sonnet — see anthropic-rate-limits-findings.md PART 3.
fn prepend_oauth_system_prefix(request: &mut MessageRequest) {
    if !model_requires_oauth_system_prefix(&request.model) {
        return;
    }
    let blocks = request.system.get_or_insert_with(Vec::new);
    let already_prefixed = blocks
        .first()
        .is_some_and(|first| first.text.starts_with(CLAUDE_CODE_OAUTH_SYSTEM_PREFIX));
    if !already_prefixed {
        blocks.insert(0, SystemBlock::ephemeral_text(CLAUDE_CODE_OAUTH_SYSTEM_PREFIX));
    }
}

fn request_id_from_headers(headers: &reqwest::header::HeaderMap) -> Option<String> {
    headers
        .get(REQUEST_ID_HEADER)
        .or_else(|| headers.get(ALT_REQUEST_ID_HEADER))
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

impl Provider for AnthropicClient {
    type Stream = MessageStream;

    fn send_message<'a>(
        &'a self,
        request: &'a MessageRequest,
    ) -> ProviderFuture<'a, MessageResponse> {
        Box::pin(async move { self.send_message(request).await })
    }

    fn stream_message<'a>(
        &'a self,
        request: &'a MessageRequest,
    ) -> ProviderFuture<'a, Self::Stream> {
        Box::pin(async move { self.stream_message(request).await })
    }
}

#[derive(Debug)]
pub struct MessageStream {
    request_id: Option<String>,
    response: reqwest::Response,
    parser: SseParser,
    pending: VecDeque<StreamEvent>,
    done: bool,
    request: MessageRequest,
    prompt_cache: Option<PromptCache>,
    latest_usage: Option<Usage>,
    usage_recorded: bool,
    last_prompt_cache_record: Arc<Mutex<Option<PromptCacheRecord>>>,
}

impl MessageStream {
    #[must_use]
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }

    pub async fn next_event(&mut self) -> Result<Option<StreamEvent>, ApiError> {
        loop {
            if let Some(event) = self.pending.pop_front() {
                self.observe_event(&event);
                return Ok(Some(event));
            }

            if self.done {
                let remaining = self.parser.finish()?;
                self.pending.extend(remaining);
                if let Some(event) = self.pending.pop_front() {
                    return Ok(Some(event));
                }
                return Ok(None);
            }

            match self.response.chunk().await? {
                Some(chunk) => {
                    self.pending.extend(self.parser.push(&chunk)?);
                }
                None => {
                    self.done = true;
                }
            }
        }
    }

    fn observe_event(&mut self, event: &StreamEvent) {
        match event {
            StreamEvent::MessageDelta(MessageDeltaEvent { usage, .. }) => {
                self.latest_usage = Some(usage.clone());
            }
            StreamEvent::MessageStop(_) => {
                if !self.usage_recorded {
                    if let (Some(prompt_cache), Some(usage)) =
                        (&self.prompt_cache, self.latest_usage.as_ref())
                    {
                        let record = prompt_cache.record_usage(&self.request, usage);
                        *self
                            .last_prompt_cache_record
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(record);
                    }
                    self.usage_recorded = true;
                }
            }
            _ => {}
        }
    }
}

async fn expect_success(response: reqwest::Response) -> Result<reqwest::Response, ApiError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    // Parse Retry-After BEFORE consuming the body (response.text() takes
    // ownership). Anthropic returns Retry-After in seconds for 429s and
    // sometimes for 503s; we accept either.
    let retry_after = response
        .headers()
        .get("retry-after")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(std::time::Duration::from_secs);

    let body = response.text().await.unwrap_or_else(|_| String::new());
    let parsed_error = serde_json::from_str::<AnthropicErrorEnvelope>(&body).ok();
    let retryable = is_retryable_status(status);

    Err(ApiError::Api {
        status,
        error_type: parsed_error
            .as_ref()
            .map(|error| error.error.error_type.clone()),
        message: parsed_error
            .as_ref()
            .map(|error| error.error.message.clone()),
        body,
        retryable,
        retry_after,
    })
}

const fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    matches!(status.as_u16(), 408 | 409 | 429 | 500 | 502 | 503 | 504)
}

#[derive(Debug, Deserialize)]
struct AnthropicErrorEnvelope {
    error: AnthropicErrorBody,
}

#[derive(Debug, Deserialize)]
struct AnthropicErrorBody {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::{ALT_REQUEST_ID_HEADER, REQUEST_ID_HEADER};
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use crate::oauth::{clear_oauth_credentials, save_oauth_credentials};
use elai_core::config::OAuthConfig;

    use super::{
        now_unix_timestamp, oauth_token_is_expired, resolve_saved_oauth_token,
        resolve_startup_auth_source, AnthropicClient, AuthSource, OAuthTokenSet,
    };
    use crate::types::{ContentBlockDelta, MessageRequest};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        crate::test_env_lock()
    }

    fn temp_config_home() -> std::path::PathBuf { // TEST_KEEP
        std::env::temp_dir().join(format!(
            "api-oauth-test-{}-{}",
            std::process::id(),
            SystemTime::now() // TEST_KEEP
                .duration_since(UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ))
    }

    fn cleanup_temp_config_home(config_home: &std::path::Path) {
        match std::fs::remove_dir_all(config_home) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => panic!("cleanup temp dir: {error}"),
        }
    }

    fn sample_oauth_config(token_url: String) -> OAuthConfig {
        OAuthConfig {
            client_id: "runtime-client".to_string(),
            authorize_url: "https://console.test/oauth/authorize".to_string(),
            token_url,
            callback_port: Some(4545),
            manual_redirect_url: Some("https://console.test/oauth/callback".to_string()),
            scopes: vec!["org:read".to_string(), "user:write".to_string()],
        }
    }

    fn spawn_token_server(response_body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
        let address = listener.local_addr().expect("local addr");
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept connection");
            let mut buffer = [0_u8; 4096];
            let _ = stream.read(&mut buffer).expect("read request");
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            stream
                .write_all(response.as_bytes())
                .expect("write response");
        });
        format!("http://{address}/oauth/token")
    }

    #[test]
    fn read_api_key_requires_presence() {
        let _guard = env_lock();
        std::env::remove_var("ANTHROPIC_AUTH_TOKEN");
        std::env::remove_var("ANTHROPIC_API_KEY");
        // Point ELAI_CONFIG_HOME at an empty tempdir so the saved-OAuth
        // lookup also returns None — otherwise the developer's real
        // ~/.elai/credentials.json bleeds in and we'd see ExpiredOAuthToken
        // instead of MissingCredentials. (patch-R1: oauth_token_is_expired
        // now flags expiresAt:null+refresh-present as expired, so the
        // saved-OAuth path is no longer silently swallowed.)
        let config_home = temp_config_home();
        std::env::set_var("ELAI_CONFIG_HOME", &config_home);
        let error = super::read_api_key().expect_err("missing key should error");
        assert!(matches!(
            error,
            crate::error::ApiError::MissingCredentials { .. }
        ), "expected MissingCredentials, got: {error:?}");
        std::env::remove_var("ELAI_CONFIG_HOME");
        cleanup_temp_config_home(&config_home);
    }

    #[test]
    fn read_api_key_requires_non_empty_value() {
        let _guard = env_lock();
        std::env::set_var("ANTHROPIC_AUTH_TOKEN", "");
        std::env::remove_var("ANTHROPIC_API_KEY");
        let config_home = temp_config_home();
        std::env::set_var("ELAI_CONFIG_HOME", &config_home);
        let error = super::read_api_key().expect_err("empty key should error");
        assert!(matches!(
            error,
            crate::error::ApiError::MissingCredentials { .. }
        ), "expected MissingCredentials, got: {error:?}");
        std::env::remove_var("ANTHROPIC_AUTH_TOKEN");
        std::env::remove_var("ELAI_CONFIG_HOME");
        cleanup_temp_config_home(&config_home);
    }

    #[test]
    fn read_api_key_prefers_api_key_env() {
        let _guard = env_lock();
        std::env::set_var("ANTHROPIC_AUTH_TOKEN", "auth-token");
        std::env::set_var("ANTHROPIC_API_KEY", "legacy-key");
        assert_eq!(
            super::read_api_key().expect("api key should load"),
            "legacy-key"
        );
        std::env::remove_var("ANTHROPIC_AUTH_TOKEN");
        std::env::remove_var("ANTHROPIC_API_KEY");
    }

    #[test]
    fn read_auth_token_reads_auth_token_env() {
        let _guard = env_lock();
        std::env::set_var("ANTHROPIC_AUTH_TOKEN", "auth-token");
        assert_eq!(super::read_auth_token().as_deref(), Some("auth-token"));
        std::env::remove_var("ANTHROPIC_AUTH_TOKEN");
    }

    #[test]
    fn oauth_token_maps_to_bearer_auth_source() {
        let auth = AuthSource::from(OAuthTokenSet {
            access_token: "access-token".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_at: Some(123),
            scopes: vec!["scope:a".to_string()],
        });
        assert_eq!(auth.bearer_token(), Some("access-token"));
        assert_eq!(auth.api_key(), None);
    }

    #[test]
    fn auth_source_from_env_combines_api_key_and_bearer_token() {
        let _guard = env_lock();
        std::env::set_var("ANTHROPIC_AUTH_TOKEN", "auth-token");
        std::env::set_var("ANTHROPIC_API_KEY", "legacy-key");
        let auth = AuthSource::from_env().expect("env auth");
        assert_eq!(auth.api_key(), Some("legacy-key"));
        assert_eq!(auth.bearer_token(), Some("auth-token"));
        std::env::remove_var("ANTHROPIC_AUTH_TOKEN");
        std::env::remove_var("ANTHROPIC_API_KEY");
    }

    #[test]
    fn auth_source_from_saved_oauth_when_env_absent() {
        let _guard = env_lock();
        let config_home = temp_config_home();
        std::env::set_var("ELAI_CONFIG_HOME", &config_home);
        std::env::remove_var("ANTHROPIC_AUTH_TOKEN");
        std::env::remove_var("ANTHROPIC_API_KEY");
        save_oauth_credentials(&crate::oauth::OAuthTokenSet {
            access_token: "saved-access-token".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_at: Some(now_unix_timestamp(&elai_core::OsClock) + 300),
            scopes: vec!["scope:a".to_string()],
        })
        .expect("save oauth credentials");

        let auth = AuthSource::from_env_or_saved().expect("saved auth");
        assert_eq!(auth.bearer_token(), Some("saved-access-token"));

        clear_oauth_credentials().expect("clear credentials");
        std::env::remove_var("ELAI_CONFIG_HOME");
        cleanup_temp_config_home(&config_home);
    }

    #[test]
    fn oauth_token_expiry_uses_expires_at_timestamp() {
        assert!(oauth_token_is_expired(&OAuthTokenSet {
            access_token: "access-token".to_string(),
            refresh_token: None,
            expires_at: Some(1),
            scopes: Vec::new(),
        }));
        assert!(!oauth_token_is_expired(&OAuthTokenSet {
            access_token: "access-token".to_string(),
            refresh_token: None,
            expires_at: Some(now_unix_timestamp(&elai_core::OsClock) + 60),
            scopes: Vec::new(),
        }));
    }

    #[test]
    fn resolve_saved_oauth_token_refreshes_expired_credentials() {
        let _guard = env_lock();
        let config_home = temp_config_home();
        std::env::set_var("ELAI_CONFIG_HOME", &config_home);
        std::env::remove_var("ANTHROPIC_AUTH_TOKEN");
        std::env::remove_var("ANTHROPIC_API_KEY");
        save_oauth_credentials(&crate::oauth::OAuthTokenSet {
            access_token: "expired-access-token".to_string(),
            refresh_token: Some("refresh-token".to_string()),
            expires_at: Some(1),
            scopes: vec!["scope:a".to_string()],
        })
        .expect("save expired oauth credentials");

        let token_url = spawn_token_server(
            "{\"access_token\":\"refreshed-token\",\"refresh_token\":\"fresh-refresh\",\"expires_at\":9999999999,\"scopes\":[\"scope:a\"]}",
        );
        let resolved = resolve_saved_oauth_token(&sample_oauth_config(token_url))
            .expect("resolve refreshed token")
            .expect("token set present");
        assert_eq!(resolved.access_token, "refreshed-token");
        let stored = crate::oauth::load_oauth_credentials()
            .expect("load stored credentials")
            .expect("stored token set");
        assert_eq!(stored.access_token, "refreshed-token");

        clear_oauth_credentials().expect("clear credentials");
        std::env::remove_var("ELAI_CONFIG_HOME");
        cleanup_temp_config_home(&config_home);
    }

    #[test]
    fn resolve_startup_auth_source_uses_saved_oauth_without_loading_config() {
        let _guard = env_lock();
        let config_home = temp_config_home();
        std::env::set_var("ELAI_CONFIG_HOME", &config_home);
        std::env::remove_var("ANTHROPIC_AUTH_TOKEN");
        std::env::remove_var("ANTHROPIC_API_KEY");
        save_oauth_credentials(&crate::oauth::OAuthTokenSet {
            access_token: "saved-access-token".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_at: Some(now_unix_timestamp(&elai_core::OsClock) + 300),
            scopes: vec!["scope:a".to_string()],
        })
        .expect("save oauth credentials");

        let auth = resolve_startup_auth_source(|| panic!("config should not be loaded"))
            .expect("startup auth");
        assert_eq!(auth.bearer_token(), Some("saved-access-token"));

        clear_oauth_credentials().expect("clear credentials");
        std::env::remove_var("ELAI_CONFIG_HOME");
        cleanup_temp_config_home(&config_home);
    }

    #[test]
    fn resolve_startup_auth_source_errors_when_refreshable_token_lacks_config() {
        let _guard = env_lock();
        let config_home = temp_config_home();
        std::env::set_var("ELAI_CONFIG_HOME", &config_home);
        std::env::remove_var("ANTHROPIC_AUTH_TOKEN");
        std::env::remove_var("ANTHROPIC_API_KEY");
        save_oauth_credentials(&crate::oauth::OAuthTokenSet {
            access_token: "expired-access-token".to_string(),
            refresh_token: Some("refresh-token".to_string()),
            expires_at: Some(1),
            scopes: vec!["scope:a".to_string()],
        })
        .expect("save expired oauth credentials");

        let error =
            resolve_startup_auth_source(|| Ok(None)).expect_err("missing config should error");
        assert!(
            matches!(error, crate::error::ApiError::Auth(message) if message.contains("runtime OAuth config is missing"))
        );

        let stored = crate::oauth::load_oauth_credentials()
            .expect("load stored credentials")
            .expect("stored token set");
        assert_eq!(stored.access_token, "expired-access-token");
        assert_eq!(stored.refresh_token.as_deref(), Some("refresh-token"));

        clear_oauth_credentials().expect("clear credentials");
        std::env::remove_var("ELAI_CONFIG_HOME");
        cleanup_temp_config_home(&config_home);
    }

    #[test]
    fn resolve_saved_oauth_token_preserves_refresh_token_when_refresh_response_omits_it() {
        let _guard = env_lock();
        let config_home = temp_config_home();
        std::env::set_var("ELAI_CONFIG_HOME", &config_home);
        std::env::remove_var("ANTHROPIC_AUTH_TOKEN");
        std::env::remove_var("ANTHROPIC_API_KEY");
        save_oauth_credentials(&crate::oauth::OAuthTokenSet {
            access_token: "expired-access-token".to_string(),
            refresh_token: Some("refresh-token".to_string()),
            expires_at: Some(1),
            scopes: vec!["scope:a".to_string()],
        })
        .expect("save expired oauth credentials");

        let token_url = spawn_token_server(
            "{\"access_token\":\"refreshed-token\",\"expires_at\":9999999999,\"scopes\":[\"scope:a\"]}",
        );
        let resolved = resolve_saved_oauth_token(&sample_oauth_config(token_url))
            .expect("resolve refreshed token")
            .expect("token set present");
        assert_eq!(resolved.access_token, "refreshed-token");
        assert_eq!(resolved.refresh_token.as_deref(), Some("refresh-token"));
        let stored = crate::oauth::load_oauth_credentials()
            .expect("load stored credentials")
            .expect("stored token set");
        assert_eq!(stored.refresh_token.as_deref(), Some("refresh-token"));

        clear_oauth_credentials().expect("clear credentials");
        std::env::remove_var("ELAI_CONFIG_HOME");
        cleanup_temp_config_home(&config_home);
    }

    #[test]
    fn message_request_stream_helper_sets_stream_true() {
        let request = MessageRequest {
            model: "claude-opus-4-6".to_string(),
            max_tokens: 64,
            messages: vec![],
            system: None,
            tools: None,
            tool_choice: None,
            stream: false,
            seed: None,
        };

        assert!(request.with_streaming().stream);
    }

    #[test]
    fn backoff_doubles_until_maximum() {
        let client = AnthropicClient::new("test-key").with_retry_policy(
            3,
            Duration::from_millis(10),
            Duration::from_millis(25),
        );
        assert_eq!(
            client.backoff_for_attempt(1).expect("attempt 1"),
            Duration::from_millis(10)
        );
        assert_eq!(
            client.backoff_for_attempt(2).expect("attempt 2"),
            Duration::from_millis(20)
        );
        assert_eq!(
            client.backoff_for_attempt(3).expect("attempt 3"),
            Duration::from_millis(25)
        );
    }

    #[test]
    fn retryable_statuses_are_detected() {
        assert!(super::is_retryable_status(
            reqwest::StatusCode::TOO_MANY_REQUESTS
        ));
        assert!(super::is_retryable_status(
            reqwest::StatusCode::INTERNAL_SERVER_ERROR
        ));
        assert!(!super::is_retryable_status(
            reqwest::StatusCode::UNAUTHORIZED
        ));
    }

    #[test]
    fn tool_delta_variant_round_trips() {
        let delta = ContentBlockDelta::InputJsonDelta {
            partial_json: "{\"city\":\"Paris\"}".to_string(),
        };
        let encoded = serde_json::to_string(&delta).expect("delta should serialize");
        let decoded: ContentBlockDelta =
            serde_json::from_str(&encoded).expect("delta should deserialize");
        assert_eq!(decoded, delta);
    }

    #[test]
    fn request_id_uses_primary_or_fallback_header() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(REQUEST_ID_HEADER, "req_primary".parse().expect("header"));
        assert_eq!(
            super::request_id_from_headers(&headers).as_deref(),
            Some("req_primary")
        );

        headers.clear();
        headers.insert(
            ALT_REQUEST_ID_HEADER,
            "req_fallback".parse().expect("header"),
        );
        assert_eq!(
            super::request_id_from_headers(&headers).as_deref(),
            Some("req_fallback")
        );
    }

    #[test]
    fn auth_source_applies_headers() {
        let auth = AuthSource::ApiKeyAndBearer {
            api_key: "test-key".to_string(),
            bearer_token: "proxy-token".to_string(),
        };
        let request = auth
            .apply(reqwest::Client::new().post("https://example.test"))
            .build()
            .expect("request build");
        let headers = request.headers();
        assert_eq!(
            headers.get("x-api-key").and_then(|v| v.to_str().ok()),
            Some("test-key")
        );
        assert_eq!(
            headers.get("authorization").and_then(|v| v.to_str().ok()),
            Some("Bearer proxy-token")
        );
    }

    // ---- patch-R1: Anthropic OAuth header & system-prefix fixes ----

    fn sample_message_request(model: &str) -> MessageRequest {
        MessageRequest {
            model: model.to_string(),
            max_tokens: 64,
            messages: vec![],
            system: None,
            tools: None,
            tool_choice: None,
            stream: false,
            seed: None,
        }
    }

    #[test]
    fn ua_constant_is_current_claude_code_version() {
        // Stale UA versions are an Anthropic abuse-detection fingerprint
        // surface. Verified 2026-05-15 against @anthropic-ai/claude-code.
        // If you bump CLAUDE_CLI_USER_AGENT, update this assertion to the
        // new version + bump cadence note in the const's doc-comment.
        assert!(
            super::CLAUDE_CLI_USER_AGENT.starts_with("claude-cli/2.1."),
            "UA must be on the 2.1.x train: {}",
            super::CLAUDE_CLI_USER_AGENT
        );
        assert!(
            super::CLAUDE_CLI_USER_AGENT.contains("(external, cli)"),
            "UA must include the published (external, cli) suffix: {}",
            super::CLAUDE_CLI_USER_AGENT
        );
    }

    #[test]
    fn oauth_system_prefix_is_required_for_opus_and_sonnet_only() {
        assert!(super::model_requires_oauth_system_prefix("claude-opus-4-7"));
        assert!(super::model_requires_oauth_system_prefix("claude-opus-4-6"));
        assert!(super::model_requires_oauth_system_prefix(
            "claude-sonnet-4-6"
        ));
        assert!(super::model_requires_oauth_system_prefix(
            "Claude-Sonnet-4-6" // case-insensitive
        ));
        assert!(!super::model_requires_oauth_system_prefix(
            "claude-haiku-4-5-20251001"
        ));
        assert!(!super::model_requires_oauth_system_prefix("qwen-coder-3b"));
        assert!(!super::model_requires_oauth_system_prefix("gpt-4"));
    }

    #[test]
    fn prepend_oauth_system_prefix_injects_on_opus_with_no_existing_system() {
        let mut req = sample_message_request("claude-opus-4-7");
        super::prepend_oauth_system_prefix(&mut req);
        let blocks = req.system.expect("system blocks should be present");
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].text, super::CLAUDE_CODE_OAUTH_SYSTEM_PREFIX);
        assert!(
            blocks[0].cache_control.is_some(),
            "prefix should be cache-marked ephemeral"
        );
    }

    #[test]
    fn prepend_oauth_system_prefix_inserts_at_index_zero_when_other_blocks_exist() {
        let mut req = sample_message_request("claude-sonnet-4-6");
        req.system = Some(vec![
            crate::SystemBlock::text("custom user system content"),
        ]);
        super::prepend_oauth_system_prefix(&mut req);
        let blocks = req.system.expect("system blocks");
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].text, super::CLAUDE_CODE_OAUTH_SYSTEM_PREFIX);
        assert_eq!(blocks[1].text, "custom user system content");
    }

    #[test]
    fn prepend_oauth_system_prefix_is_idempotent() {
        let mut req = sample_message_request("claude-opus-4-7");
        super::prepend_oauth_system_prefix(&mut req);
        super::prepend_oauth_system_prefix(&mut req);
        super::prepend_oauth_system_prefix(&mut req);
        let blocks = req.system.expect("system blocks");
        assert_eq!(blocks.len(), 1, "must not double-prepend on repeat calls");
    }

    #[test]
    fn prepend_oauth_system_prefix_skips_haiku() {
        let mut req = sample_message_request("claude-haiku-4-5-20251001");
        super::prepend_oauth_system_prefix(&mut req);
        assert!(
            req.system.is_none(),
            "Haiku should not get the OAuth system prefix"
        );
    }

    #[test]
    fn prepend_oauth_system_prefix_recognises_existing_prefix_block() {
        let mut req = sample_message_request("claude-sonnet-4-6");
        req.system = Some(vec![
            crate::SystemBlock::ephemeral_text(
                super::CLAUDE_CODE_OAUTH_SYSTEM_PREFIX,
            ),
            crate::SystemBlock::text("user content"),
        ]);
        super::prepend_oauth_system_prefix(&mut req);
        let blocks = req.system.expect("system blocks");
        assert_eq!(blocks.len(), 2, "must not duplicate existing prefix block");
    }

    #[test]
    fn oauth_token_is_expired_returns_true_when_expires_at_none_and_refresh_present() {
        // The recurring "expiresAt: null" short-circuit bug from
        // PHASE_E2_REPORT — once-saved tokens with null expiry must be
        // treated as expired so the refresh path fires.
        let token = OAuthTokenSet {
            access_token: "stale-access".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_at: None,
            scopes: vec![],
        };
        assert!(super::oauth_token_is_expired(&token));
    }

    #[test]
    fn oauth_token_is_expired_returns_false_when_expires_at_none_and_no_refresh() {
        // Without a refresh token there's nothing we can do; preserve
        // prior behaviour and let the live request surface the 401.
        let token = OAuthTokenSet {
            access_token: "static".to_string(),
            refresh_token: None,
            expires_at: None,
            scopes: vec![],
        };
        assert!(!super::oauth_token_is_expired(&token));
    }

    #[test]
    fn oauth_token_is_expired_uses_expires_at_when_present() {
        let now = super::now_unix_timestamp(&elai_core::OsClock);
        let expired = OAuthTokenSet {
            access_token: "x".to_string(),
            refresh_token: None,
            expires_at: Some(now.saturating_sub(60)),
            scopes: vec![],
        };
        let valid = OAuthTokenSet {
            access_token: "x".to_string(),
            refresh_token: None,
            expires_at: Some(now + 3600),
            scopes: vec![],
        };
        assert!(super::oauth_token_is_expired(&expired));
        assert!(!super::oauth_token_is_expired(&valid));
    }

    #[test]
    fn oauth_usage_should_throttle_at_or_above_threshold() {
        let safe = super::OAuthUsage {
            five_hour: 50.0,
            seven_day: 60.0,
        };
        let near_cap_5h = super::OAuthUsage {
            five_hour: 85.0,
            seven_day: 60.0,
        };
        let near_cap_7d = super::OAuthUsage {
            five_hour: 50.0,
            seven_day: 81.0,
        };
        assert!(!safe.should_throttle(80.0));
        assert!(near_cap_5h.should_throttle(80.0));
        assert!(near_cap_7d.should_throttle(80.0));
    }

    #[test]
    fn fetch_oauth_usage_requires_bearer_token_auth() {
        let _guard = env_lock();
        let client = AnthropicClient::new("test-api-key");
        // Tokio runtime needed because fetch_oauth_usage is async.
        let result = tokio::runtime::Runtime::new()
            .expect("runtime")
            .block_on(client.fetch_oauth_usage());
        match result {
            Err(crate::ApiError::Auth(msg)) => {
                assert!(
                    msg.contains("BearerToken"),
                    "auth error should mention BearerToken requirement: {msg}"
                );
            }
            other => panic!("expected ApiError::Auth, got: {other:?}"),
        }
    }
}
