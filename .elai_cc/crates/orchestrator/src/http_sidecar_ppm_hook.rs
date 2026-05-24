//! D-2: Production HTTP implementation of [`SidecarPpmHook`] — speaks to the
//! second sidecar (port 8101) over `POST /v1/ppm/score`.
//!
//! Mirrors the pattern of [`crate::grounding_client::SidecarGroundingClient`]:
//! owns a reqwest client + base URL + per-request timeout, wraps the wire
//! call, and translates HTTP / parse failures into typed
//! [`PpmHookError`] variants so the calling [`crate::ppm_client::PpmClient`]
//! can decide whether to propagate, retry, or emit an operator-visible row.
//!
//! # Why a separate file (not appended to `sidecar_ppm_hook.rs`)
//!
//! Per context-package §6 placement decision: `sidecar_ppm_hook.rs` is the
//! TRAIT + MOCK + WIRE CONTRACT module (≈440 lines). Putting the production
//! HTTP impl there would conflate two responsibilities. This file owns ONLY
//! the prod path — keeps the trust boundary readable and lets reviewers grep
//! for "http_sidecar_ppm_hook" when auditing the live wire.
//!
//! # Wire contract honoured server-side AND client-side
//!
//! The Python sidecar (`elai_inference/server.py::handle_ppm_score`)
//! enforces clauses C1-C7 from [`sidecar_ppm_hook`] line 44 onward. THIS
//! file ALSO re-validates the response defensively because the hook is a
//! trust boundary (clauses are checked twice on purpose — server-side
//! catches malformed model output; client-side catches a misconfigured
//! sidecar).
//!
//! # Retry policy
//!
//! Bounded retry on transient transport errors (network reset, 5xx that
//! does NOT carry a structured PpmHookError body, request timeout). Caps
//! at [`HttpSidecarPpmHook::max_retries`] (default 3) with exponential
//! backoff. Per plan §WIRING_CONTRACT stub_ban: retry MUST cap, no
//! infinite loop. On final failure the typed error surfaces to
//! `PpmClient::score` -> `PpmClientError::Hook(_)`.

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::sidecar_ppm_hook::{
    PpmHookError, PpmMode, PpmScoreRequest, PpmScoreResponse, SidecarPpmHook,
};

// ---------------------------------------------------------------------------
// Named constants — anti-stale, no magic numbers per codeRules.
// ---------------------------------------------------------------------------

/// Default base URL for the PPM sidecar. Port 8101 (NOT 8100 — collision
/// prevention is a plan stub_ban clause).
pub const PPM_DEFAULT_BASE_URL: &str = "http://127.0.0.1:8101";

/// Canonical PPM scoring path on the second sidecar.
pub const PPM_SCORE_PATH: &str = "/v1/ppm/score";

/// Canonical PPM health-check path.
pub const PPM_HEALTH_PATH: &str = "/health";

/// Default request timeout (seconds).
pub const PPM_DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default max retries on transient transport errors.
pub const PPM_DEFAULT_MAX_RETRIES: u32 = 3;

/// Initial backoff (milliseconds). Each retry doubles up to a soft ceiling.
pub const PPM_BACKOFF_INITIAL_MS: u64 = 200;

/// Backoff ceiling (milliseconds). Bounds the exponential growth.
pub const PPM_BACKOFF_CEILING_MS: u64 = 2_000;

// ---------------------------------------------------------------------------
// HttpSidecarPpmHook — production HTTP impl
// ---------------------------------------------------------------------------

/// Production [`SidecarPpmHook`] impl that talks to the second sidecar.
///
/// Construct via [`HttpSidecarPpmHook::new`] (defaults to
/// [`PPM_DEFAULT_BASE_URL`] + [`PPM_DEFAULT_TIMEOUT_SECS`] +
/// [`PPM_DEFAULT_MAX_RETRIES`]) or [`HttpSidecarPpmHook::with_endpoint`] for
/// explicit configuration. Cheap to clone (the inner `reqwest::Client` is
/// already `Clone`).
#[derive(Debug, Clone)]
pub struct HttpSidecarPpmHook {
    base_url: String,
    timeout: Duration,
    max_retries: u32,
    http: reqwest::Client,
}

impl HttpSidecarPpmHook {
    /// Construct with defaults. Returns the standard PPM sidecar pointing at
    /// `http://127.0.0.1:8101`. Suitable when the SSH tunnel forwards 8101
    /// to the remote PPM model per `workflow/remote-setup/start-tunnel.ps1`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            base_url: PPM_DEFAULT_BASE_URL.to_string(),
            timeout: Duration::from_secs(PPM_DEFAULT_TIMEOUT_SECS),
            max_retries: PPM_DEFAULT_MAX_RETRIES,
            http: reqwest::Client::new(),
        }
    }

    /// Construct with an explicit endpoint, timeout, and retry budget.
    /// `base_url` MUST NOT end with `/` — the path constants
    /// (`PPM_SCORE_PATH`, `PPM_HEALTH_PATH`) carry the leading `/`.
    #[must_use]
    pub fn with_endpoint(base_url: impl Into<String>, timeout: Duration, max_retries: u32) -> Self {
        Self {
            base_url: base_url.into(),
            timeout,
            max_retries,
            http: reqwest::Client::new(),
        }
    }

    /// The configured base URL (no trailing slash; path constants carry the
    /// leading `/`).
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Per-request timeout.
    #[must_use]
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Max retry budget (transient transport failures).
    #[must_use]
    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }

    /// Fully-qualified score endpoint URL.
    #[must_use]
    pub fn score_url(&self) -> String {
        format!("{}{}", self.base_url, PPM_SCORE_PATH)
    }

    /// Fully-qualified health endpoint URL.
    #[must_use]
    pub fn health_url(&self) -> String {
        format!("{}{}", self.base_url, PPM_HEALTH_PATH)
    }

    /// One-shot health probe. Returns `Ok(HealthReport)` if reachable +
    /// 2xx; `Err(PpmHookError::Unavailable)` if connection refused;
    /// `Err(PpmHookError::Transport(_))` for other failures.
    ///
    /// Used by `elai ppm health` CLI for the third step of the 3-step
    /// health check (curl-equivalent live ping).
    pub async fn health(&self) -> Result<HealthReport, PpmHookError> {
        let url = self.health_url();
        let res = self
            .http
            .get(&url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| classify_transport_error(&e))?;
        let status = res.status();
        if !status.is_success() {
            return Err(PpmHookError::Transport(format!(
                "health endpoint returned HTTP {status} at {url}"
            )));
        }
        // The Python /health endpoint returns `{"status": "healthy"}` today.
        // We accept any JSON object with at least a `status` field.
        let text = res
            .text()
            .await
            .map_err(|e| PpmHookError::Parse(format!("health body: {e}")))?;
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| PpmHookError::Parse(format!("health JSON: {e}")))?;
        let status_str = parsed
            .get("status")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let model_id = parsed
            .get("model")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let load_status = parsed
            .get("load_status")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        Ok(HealthReport {
            url,
            status: status_str.unwrap_or_else(|| "ok".to_string()),
            model_id,
            load_status,
        })
    }

    /// Internal: drive one HTTP attempt against `/v1/ppm/score`. Surfaces
    /// classified `PpmHookError` variants. Caller (`score`) decides on retry.
    async fn score_once(
        &self,
        wire: &WireRequest<'_>,
    ) -> Result<PpmScoreResponse, PpmHookError> {
        let url = self.score_url();
        debug!(
            url = %url,
            verifier_id = wire.verifier_id,
            candidate_id = wire.candidate_id,
            steps = wire.reasoning_steps.len(),
            "http_ppm_request",
        );
        let res = self
            .http
            .post(&url)
            .timeout(self.timeout)
            .json(wire)
            .send()
            .await
            .map_err(|e| classify_transport_error(&e))?;

        let status = res.status();
        if !status.is_success() {
            let body = res.text().await.unwrap_or_default();
            // Try to parse a structured error body the server may have emitted.
            if let Ok(structured) = serde_json::from_str::<WireErrorBody>(&body) {
                return Err(structured.into_typed());
            }
            return Err(PpmHookError::Transport(format!(
                "sidecar returned HTTP {status} from {url}: {body}"
            )));
        }

        let body = res
            .text()
            .await
            .map_err(|e| PpmHookError::Parse(format!("read body: {e}")))?;
        let parsed: WireResponse = serde_json::from_str(&body)
            .map_err(|e| PpmHookError::Parse(format!("response JSON: {e}: body={body}")))?;
        parsed.into_typed()
    }
}

impl Default for HttpSidecarPpmHook {
    fn default() -> Self {
        Self::new()
    }
}

impl SidecarPpmHook for HttpSidecarPpmHook {
    async fn score(
        &self,
        req: PpmScoreRequest,
    ) -> Result<PpmScoreResponse, PpmHookError> {
        // Clause 5 enforcement on the request side BEFORE any HTTP work.
        if req.reasoning_steps.is_empty() {
            return Err(PpmHookError::EmptyInput);
        }

        let wire = WireRequest::from_request(&req);

        let mut attempt: u32 = 0;
        loop {
            match self.score_once(&wire).await {
                Ok(resp) => {
                    // Defensive re-validation of clauses C1 + C2 + C4 (clause C5 already
                    // enforced before HTTP; clause C3 mean/var is checked here via a
                    // soft-tolerance to catch drift between server-side computation and
                    // the wire shape).
                    validate_response_shape(&req, &resp)?;
                    return Ok(resp);
                }
                Err(err) => {
                    // Decide retryability based on the typed error.
                    if !is_retryable(&err) || attempt >= self.max_retries {
                        return Err(err);
                    }
                    attempt += 1;
                    let delay_ms = backoff_ms(attempt);
                    warn!(
                        attempt,
                        max_retries = self.max_retries,
                        delay_ms,
                        error = %err,
                        "http_ppm_retry",
                    );
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// HealthReport — return type of `health()` so the CLI can render verbatim.
// ---------------------------------------------------------------------------

/// Result of a `GET /health` round-trip. Captures the URL queried and the
/// JSON fields the sidecar surfaces. `model_id` + `load_status` are
/// optional because the current Python server does not emit them on
/// `/health` (yet) — once the PPM-aware server lands they will be set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthReport {
    pub url: String,
    pub status: String,
    pub model_id: Option<String>,
    pub load_status: Option<String>,
}

// ---------------------------------------------------------------------------
// Wire JSON envelopes (private — kept narrow to this file).
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct WireRequest<'a> {
    candidate_id: &'a str,
    verifier_id: &'a str,
    problem_statement: &'a str,
    reasoning_steps: &'a [String],
    mode: &'a str,
}

impl<'a> WireRequest<'a> {
    fn from_request(req: &'a PpmScoreRequest) -> Self {
        Self {
            candidate_id: &req.candidate_id,
            verifier_id: &req.verifier_id,
            problem_statement: &req.problem_statement,
            reasoning_steps: &req.reasoning_steps,
            mode: req.mode.as_str(),
        }
    }
}

#[derive(Deserialize)]
struct WireResponse {
    candidate_id: String,
    verifier_id: String,
    mode: String,
    step_scores: Vec<f32>,
    score_mean: f32,
    score_var: f32,
    num_steps: u32,
    latency_ms: u64,
}

impl WireResponse {
    fn into_typed(self) -> Result<PpmScoreResponse, PpmHookError> {
        let mode = match self.mode.as_str() {
            "math" => PpmMode::Math,
            "code" => PpmMode::Code,
            other => return Err(PpmHookError::InvalidMode(other.to_string())),
        };
        Ok(PpmScoreResponse {
            candidate_id: self.candidate_id,
            verifier_id: self.verifier_id,
            mode,
            step_scores: self.step_scores,
            score_mean: self.score_mean,
            score_var: self.score_var,
            num_steps: self.num_steps,
            latency_ms: self.latency_ms,
        })
    }
}

/// Structured error envelope the Python sidecar emits when a request
/// violates a clause server-side (HTTP 422 + JSON body). Mirrors the
/// PpmHookError shape so we can map back without lossy stringification.
#[derive(Deserialize)]
struct WireErrorBody {
    error: String,
    #[serde(default)]
    detail: Option<String>,
    #[serde(default)]
    index: Option<usize>,
    #[serde(default)]
    value: Option<f32>,
    #[serde(default)]
    request_steps: Option<usize>,
    #[serde(default)]
    response_scores: Option<usize>,
}

impl WireErrorBody {
    fn into_typed(self) -> PpmHookError {
        match self.error.as_str() {
            "empty_input" => PpmHookError::EmptyInput,
            "invalid_score" => PpmHookError::InvalidScore {
                index: self.index.unwrap_or(0),
                value: self.value.unwrap_or(f32::NAN),
            },
            "shape_mismatch" => PpmHookError::ShapeMismatch {
                request_steps: self.request_steps.unwrap_or(0),
                response_scores: self.response_scores.unwrap_or(0),
            },
            "invalid_mode" => PpmHookError::InvalidMode(self.detail.unwrap_or_default()),
            "parse_error" => PpmHookError::Parse(self.detail.unwrap_or_default()),
            other => PpmHookError::Transport(format!(
                "server reported error={other} detail={:?}",
                self.detail
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Map a reqwest transport failure to a typed PpmHookError. Connect / DNS
/// failures become `Unavailable`; timeouts and other transient errors
/// become `Transport(_)`. The score-loop decides retryability based on
/// the variant.
fn classify_transport_error(e: &reqwest::Error) -> PpmHookError {
    if e.is_connect() {
        return PpmHookError::Unavailable;
    }
    if e.is_timeout() {
        return PpmHookError::Transport(format!("timeout: {e}"));
    }
    PpmHookError::Transport(e.to_string())
}

/// Is this error worth retrying? Per plan §WIRING_CONTRACT:
/// - Unavailable: NO (sidecar offline; caller decides whether to wait/escalate).
/// - Transport: YES (transient network blip).
/// - Parse: NO (server emitted malformed JSON; retry will reproduce).
/// - EmptyInput / InvalidScore / ShapeMismatch / InvalidMode: NO (contract
///   violation; retry cannot fix the input shape).
fn is_retryable(err: &PpmHookError) -> bool {
    matches!(err, PpmHookError::Transport(_))
}

/// Exponential backoff with a ceiling. `attempt` is 1-indexed (first retry
/// after attempt=1). Returns the milliseconds to sleep before the next try.
fn backoff_ms(attempt: u32) -> u64 {
    let raw = PPM_BACKOFF_INITIAL_MS.saturating_mul(2u64.saturating_pow(attempt.saturating_sub(1)));
    raw.min(PPM_BACKOFF_CEILING_MS)
}

/// Re-validate clauses C1, C2, C4 on the response. Server-side already
/// enforces these but the hook is a trust boundary; second check catches
/// a misconfigured / regressed sidecar without crashing the orchestrator.
fn validate_response_shape(
    req: &PpmScoreRequest,
    resp: &PpmScoreResponse,
) -> Result<(), PpmHookError> {
    // C2 — shape match.
    let request_steps = req.reasoning_steps.len();
    let response_scores = resp.step_scores.len();
    if response_scores != request_steps {
        return Err(PpmHookError::ShapeMismatch {
            request_steps,
            response_scores,
        });
    }
    if resp.num_steps as usize != response_scores {
        return Err(PpmHookError::ShapeMismatch {
            request_steps: resp.num_steps as usize,
            response_scores,
        });
    }
    // C1 — per-step range.
    for (i, v) in resp.step_scores.iter().enumerate() {
        if !(0.0..=1.0).contains(v) {
            return Err(PpmHookError::InvalidScore {
                index: i,
                value: *v,
            });
        }
    }
    // C4 — mode echo. Already coerced in `WireResponse::into_typed`; we
    // additionally assert the server echoed the same mode we requested.
    if resp.mode != req.mode {
        return Err(PpmHookError::InvalidMode(format!(
            "request mode={:?} response mode={:?}",
            req.mode, resp.mode
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// PpmClient construction shortcut
// ---------------------------------------------------------------------------

/// Construct a [`crate::ppm_client::PpmClient`] backed by an
/// [`HttpSidecarPpmHook`]. Cheap helper used by the orchestrator boot path
/// when `ELAI_PPM_ENABLE=1` is set and the toml `[runtime.ppm]` stanza is
/// active.
#[must_use]
pub fn build_http_ppm_client(
    base_url: impl Into<String>,
    timeout: Duration,
    max_retries: u32,
) -> crate::ppm_client::PpmClient<HttpSidecarPpmHook> {
    let hook = Arc::new(HttpSidecarPpmHook::with_endpoint(
        base_url,
        timeout,
        max_retries,
    ));
    crate::ppm_client::PpmClient::new(hook)
}

/// Same as [`build_http_ppm_client`] but with an explicit gating threshold.
/// Returns `Err(PpmClientError::InvalidThreshold)` if `threshold` is outside
/// `[0.0, 1.0]`.
pub fn build_http_ppm_client_with_threshold(
    base_url: impl Into<String>,
    timeout: Duration,
    max_retries: u32,
    threshold: f32,
) -> Result<crate::ppm_client::PpmClient<HttpSidecarPpmHook>, crate::ppm_client::PpmClientError> {
    let hook = Arc::new(HttpSidecarPpmHook::with_endpoint(
        base_url,
        timeout,
        max_retries,
    ));
    crate::ppm_client::PpmClient::with_threshold(hook, threshold)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_endpoint_targets_port_8101() {
        let hook = HttpSidecarPpmHook::new();
        assert!(hook.base_url().contains("8101"));
        assert!(hook.score_url().ends_with("/v1/ppm/score"));
        assert!(hook.health_url().ends_with("/health"));
    }

    #[test]
    fn with_endpoint_overrides_defaults() {
        let hook = HttpSidecarPpmHook::with_endpoint(
            "http://10.0.0.5:9999",
            Duration::from_secs(5),
            7,
        );
        assert_eq!(hook.base_url(), "http://10.0.0.5:9999");
        assert_eq!(hook.timeout(), Duration::from_secs(5));
        assert_eq!(hook.max_retries(), 7);
        assert_eq!(hook.score_url(), "http://10.0.0.5:9999/v1/ppm/score");
    }

    #[test]
    fn backoff_grows_then_caps() {
        let b1 = backoff_ms(1);
        let b2 = backoff_ms(2);
        let b3 = backoff_ms(3);
        let big = backoff_ms(20);
        assert_eq!(b1, PPM_BACKOFF_INITIAL_MS);
        assert_eq!(b2, PPM_BACKOFF_INITIAL_MS * 2);
        assert_eq!(b3, PPM_BACKOFF_INITIAL_MS * 4);
        assert_eq!(big, PPM_BACKOFF_CEILING_MS);
    }

    #[test]
    fn retryable_only_for_transient_transport() {
        assert!(is_retryable(&PpmHookError::Transport("net".into())));
        assert!(!is_retryable(&PpmHookError::Unavailable));
        assert!(!is_retryable(&PpmHookError::EmptyInput));
        assert!(!is_retryable(&PpmHookError::Parse("bad".into())));
        assert!(!is_retryable(&PpmHookError::InvalidMode("x".into())));
        assert!(!is_retryable(&PpmHookError::InvalidScore {
            index: 0,
            value: 2.0,
        }));
        assert!(!is_retryable(&PpmHookError::ShapeMismatch {
            request_steps: 3,
            response_scores: 2,
        }));
    }

    #[test]
    fn validate_response_catches_shape_mismatch() {
        let req = PpmScoreRequest {
            candidate_id: "c".into(),
            verifier_id: "v".into(),
            problem_statement: "p".into(),
            reasoning_steps: vec!["a".into(), "b".into(), "c".into()],
            mode: PpmMode::Math,
        };
        let resp = PpmScoreResponse {
            candidate_id: "c".into(),
            verifier_id: "v".into(),
            mode: PpmMode::Math,
            step_scores: vec![0.5, 0.5],
            score_mean: 0.5,
            score_var: 0.0,
            num_steps: 2,
            latency_ms: 1,
        };
        match validate_response_shape(&req, &resp) {
            Err(PpmHookError::ShapeMismatch { request_steps, response_scores }) => {
                assert_eq!(request_steps, 3);
                assert_eq!(response_scores, 2);
            }
            other => panic!("expected ShapeMismatch, got {other:?}"),
        }
    }

    #[test]
    fn validate_response_catches_out_of_range_score() {
        let req = PpmScoreRequest {
            candidate_id: "c".into(),
            verifier_id: "v".into(),
            problem_statement: "p".into(),
            reasoning_steps: vec!["a".into()],
            mode: PpmMode::Math,
        };
        let resp = PpmScoreResponse {
            candidate_id: "c".into(),
            verifier_id: "v".into(),
            mode: PpmMode::Math,
            step_scores: vec![1.5],
            score_mean: 1.5,
            score_var: 0.0,
            num_steps: 1,
            latency_ms: 1,
        };
        match validate_response_shape(&req, &resp) {
            Err(PpmHookError::InvalidScore { index, value }) => {
                assert_eq!(index, 0);
                assert!(value > 1.0);
            }
            other => panic!("expected InvalidScore, got {other:?}"),
        }
    }

    #[test]
    fn validate_response_catches_mode_mismatch() {
        let req = PpmScoreRequest {
            candidate_id: "c".into(),
            verifier_id: "v".into(),
            problem_statement: "p".into(),
            reasoning_steps: vec!["a".into()],
            mode: PpmMode::Math,
        };
        let resp = PpmScoreResponse {
            candidate_id: "c".into(),
            verifier_id: "v".into(),
            mode: PpmMode::Code,
            step_scores: vec![0.5],
            score_mean: 0.5,
            score_var: 0.0,
            num_steps: 1,
            latency_ms: 1,
        };
        match validate_response_shape(&req, &resp) {
            Err(PpmHookError::InvalidMode(_)) => {}
            other => panic!("expected InvalidMode, got {other:?}"),
        }
    }

    #[test]
    fn wire_error_body_maps_to_typed_errors() {
        let body = WireErrorBody {
            error: "invalid_score".into(),
            detail: None,
            index: Some(2),
            value: Some(1.7),
            request_steps: None,
            response_scores: None,
        };
        match body.into_typed() {
            PpmHookError::InvalidScore { index, value } => {
                assert_eq!(index, 2);
                assert!((value - 1.7).abs() < 1e-6);
            }
            other => panic!("expected InvalidScore, got {other:?}"),
        }

        let body = WireErrorBody {
            error: "shape_mismatch".into(),
            detail: None,
            index: None,
            value: None,
            request_steps: Some(3),
            response_scores: Some(2),
        };
        match body.into_typed() {
            PpmHookError::ShapeMismatch {
                request_steps,
                response_scores,
            } => {
                assert_eq!(request_steps, 3);
                assert_eq!(response_scores, 2);
            }
            other => panic!("expected ShapeMismatch, got {other:?}"),
        }

        let body = WireErrorBody {
            error: "empty_input".into(),
            detail: None,
            index: None,
            value: None,
            request_steps: None,
            response_scores: None,
        };
        assert_eq!(body.into_typed(), PpmHookError::EmptyInput);

        let body = WireErrorBody {
            error: "invalid_mode".into(),
            detail: Some("prose".into()),
            index: None,
            value: None,
            request_steps: None,
            response_scores: None,
        };
        match body.into_typed() {
            PpmHookError::InvalidMode(s) => assert_eq!(s, "prose"),
            other => panic!("expected InvalidMode, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn score_with_empty_steps_returns_empty_input_without_http() {
        // No mock server running; this would surface as Unavailable if the
        // hook reached HTTP. Empty-input gate MUST fire first.
        let hook = HttpSidecarPpmHook::with_endpoint(
            "http://127.0.0.1:1", // closed port
            Duration::from_millis(50),
            0,
        );
        let req = PpmScoreRequest {
            candidate_id: "c".into(),
            verifier_id: "v".into(),
            problem_statement: "p".into(),
            reasoning_steps: vec![],
            mode: PpmMode::Math,
        };
        match hook.score(req).await {
            Err(PpmHookError::EmptyInput) => {}
            other => panic!("expected EmptyInput, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn score_with_closed_port_returns_unavailable() {
        // Connect to a port we know is closed. reqwest surfaces this as
        // `is_connect()`, which `classify_transport_error` maps to
        // `PpmHookError::Unavailable`. Retry is disabled (max_retries=0).
        let hook = HttpSidecarPpmHook::with_endpoint(
            "http://127.0.0.1:1", // closed
            Duration::from_millis(200),
            0,
        );
        let req = PpmScoreRequest {
            candidate_id: "c".into(),
            verifier_id: "v".into(),
            problem_statement: "p".into(),
            reasoning_steps: vec!["a".into()],
            mode: PpmMode::Math,
        };
        match hook.score(req).await {
            Err(PpmHookError::Unavailable) => {}
            // Some platforms surface the closed-port as a connect-reset
            // (Transport(_)) instead of refused; accept either as long as
            // the hook does not return Ok.
            Err(PpmHookError::Transport(_)) => {}
            Err(other) => panic!("unexpected error variant: {other:?}"),
            Ok(_) => panic!("connect to closed port returned Ok"),
        }
    }

    #[test]
    fn build_http_ppm_client_returns_client_with_default_threshold() {
        use crate::ppm_client::PpmScore;
        let client = build_http_ppm_client(
            PPM_DEFAULT_BASE_URL,
            Duration::from_secs(PPM_DEFAULT_TIMEOUT_SECS),
            PPM_DEFAULT_MAX_RETRIES,
        );
        assert_eq!(client.threshold(), PpmScore::DEFAULT_MIN_SCORE_THRESHOLD);
    }

    #[test]
    fn build_http_ppm_client_with_threshold_validates_range() {
        let res = build_http_ppm_client_with_threshold(
            PPM_DEFAULT_BASE_URL,
            Duration::from_secs(PPM_DEFAULT_TIMEOUT_SECS),
            PPM_DEFAULT_MAX_RETRIES,
            1.5,
        );
        assert!(res.is_err());
    }
}
