//! H-5 — Anthropic Message Batches API client for non-interactive lanes.
//!
//! The Batch API bills 50% of the synchronous price on every model with zero
//! quality cost, in exchange for an asynchronous (<=24h) completion window. It is
//! therefore the right transport for AUTONOMOUS, non-human-waited LLM work (the
//! cosmos refresh research fan-out), and the WRONG transport for any human-waited
//! call (interactive `/task`, chat REPL, the dashboard refresh button) — those stay
//! on the synchronous streaming path.
//!
//! GD-1 (compose, do not restate): this is a NEW sibling to [`super::anthropic`], not
//! a replacement. It reuses [`AuthSource`] (auth header application) and
//! [`AnthropicRequestProfile::render_json_body`] (request serialization) from the
//! synchronous client rather than re-implementing the transport.
//!
//! Grounding (verify:batch-api, re-verified 2026-06-04 — see workflow/reasoning/H-5-grounding.md):
//!   * endpoint  POST /v1/messages/batches ; GET .../{id} ; GET .../{id}/results ; POST .../{id}/cancel
//!   * GA — header `anthropic-version: 2023-06-01` only, NO beta header
//!   * 50% pricing input AND output, all models ; 24h window / 29d retention / 100k-or-256MB cap
//!   * batch items EXCLUDE streaming + Fast mode + max_tokens:0 ; support tools + system + thinking
//!   * results return OUT OF SUBMISSION ORDER — every item carries a `custom_id` for re-mapping
//!
//! Auth lane: the non-interactive batch path runs on the server API key. OAuth-bearer
//! is the interactive lane (which never batches), so the OAuth system-prefix munging in
//! [`super::anthropic`] is intentionally out of scope here.

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;

use super::anthropic::{AuthSource, DEFAULT_BASE_URL};
use crate::error::ApiError;
use crate::telemetry_types::AnthropicRequestProfile;
use crate::types::{MessageRequest, MessageResponse};

/// Message Batches is GA: only the dated version header is required (no beta).
const BATCH_ANTHROPIC_VERSION: &str = "2023-06-01";

/// HD-1 — the 50%-off Batch price expressed as a journal-recomputable half of the
/// synchronous cents. Floor division (never rounds the bill UP). Consumed by
/// `BatchLifecyclePayload.spend_cents_half` so the dashboard cost panel can prove the
/// saving from the SSB stream under a pinned seed (GD-2).
#[must_use]
pub fn half_price_cents(full_cents: i64) -> i64 {
    full_cents / 2
}

/// One request slot in a batch submission. `custom_id` round-trips so the
/// out-of-order results can be mapped back to their source unit (artifact / node).
#[derive(Debug, Clone)]
pub struct BatchItem {
    /// Caller-stable id echoed back on the matching result line.
    pub custom_id: String,
    /// The synchronous-shape message request (streaming is forced off at submit).
    pub request: MessageRequest,
}

/// Server-assigned batch id (`msgbatch_...`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchId(pub String);

/// Lifecycle status returned by [`AnthropicBatchClient::poll`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchProcessingStatus {
    /// At least one request is still processing.
    InProgress,
    /// All requests reached a terminal state; results are retrievable.
    Ended,
    /// A cancel was requested and is propagating.
    Canceling,
}

/// Decoded poll snapshot with per-request terminal counts.
#[derive(Debug, Clone)]
pub struct BatchStatus {
    /// The batch this status belongs to.
    pub id: BatchId,
    /// Coarse processing status.
    pub processing_status: BatchProcessingStatus,
    /// Requests still processing.
    pub processing: u32,
    /// Requests that succeeded.
    pub succeeded: u32,
    /// Requests that errored.
    pub errored: u32,
    /// Requests canceled.
    pub canceled: u32,
    /// Requests that expired before processing (24h window elapsed).
    pub expired: u32,
}

impl BatchStatus {
    /// True once all requests reached a terminal state (results retrievable).
    #[must_use]
    pub fn is_ended(&self) -> bool {
        matches!(self.processing_status, BatchProcessingStatus::Ended)
    }
}

/// Per-request terminal outcome from [`AnthropicBatchClient::retrieve`].
#[derive(Debug)]
pub enum BatchResultOutcome {
    /// The request completed; carries the standard message response.
    Succeeded(Box<MessageResponse>),
    /// The request errored; carries the raw error object for diagnostics.
    Errored(Value),
    /// The request was canceled before completion.
    Canceled,
    /// The request expired (24h window elapsed without processing).
    Expired,
}

/// One line of the JSONL results stream, keyed by the submitted `custom_id`.
#[derive(Debug)]
pub struct BatchResultItem {
    /// The `custom_id` from the corresponding [`BatchItem`].
    pub custom_id: String,
    /// The terminal outcome for that request.
    pub outcome: BatchResultOutcome,
}

/// Async client for the Anthropic Message Batches API. Mirrors the construction
/// surface of [`super::anthropic::AnthropicClient`] (new / from_auth / with_base_url)
/// and reuses its auth + request-rendering pieces (GD-1).
pub struct AnthropicBatchClient {
    http: reqwest::Client,
    auth: AuthSource,
    base_url: String,
    request_profile: AnthropicRequestProfile,
}

impl AnthropicBatchClient {
    /// Construct with a raw API key (the non-interactive server-credential lane).
    #[must_use]
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            auth: AuthSource::ApiKey(api_key.into()),
            base_url: DEFAULT_BASE_URL.to_string(),
            request_profile: AnthropicRequestProfile::default(),
        }
    }

    /// Construct from an explicit [`AuthSource`] (shares the synchronous client's enum).
    #[must_use]
    pub fn from_auth(auth: AuthSource) -> Self {
        Self {
            http: reqwest::Client::new(),
            auth,
            base_url: DEFAULT_BASE_URL.to_string(),
            request_profile: AnthropicRequestProfile::default(),
        }
    }

    /// Override the base URL (e.g. a custom `ANTHROPIC_BASE_URL` or a test fixture).
    #[must_use]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// The GA version header value the client sends. Stable; no beta header.
    #[must_use]
    pub fn anthropic_version(&self) -> &'static str {
        BATCH_ANTHROPIC_VERSION
    }

    fn batches_url(&self) -> String {
        format!("{}/v1/messages/batches", self.base_url.trim_end_matches('/'))
    }

    fn authed(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        self.auth
            .apply(builder)
            .header("content-type", "application/json")
            .header("anthropic-version", BATCH_ANTHROPIC_VERSION)
    }

    /// Submit a batch of message requests. Streaming is forced OFF on every item
    /// (the Batch API rejects `stream:true`). Returns the server batch id.
    pub async fn create(&self, items: &[BatchItem]) -> Result<BatchId, ApiError> {
        let mut requests = Vec::with_capacity(items.len());
        for item in items {
            // Excluded params: streaming is forced off; Fast mode / max_tokens:0 are
            // the caller's responsibility not to set on a batch item (the cosmos lane
            // never does). Reuse the synchronous serializer (GD-1).
            let req = MessageRequest {
                stream: false,
                ..item.request.clone()
            };
            let params = self.request_profile.render_json_body(&req)?;
            requests.push(serde_json::json!({
                "custom_id": item.custom_id,
                "params": params,
            }));
        }
        let body = serde_json::json!({ "requests": requests });
        let resp = self
            .authed(self.http.post(&self.batches_url()))
            .json(&body)
            .send()
            .await
            .map_err(ApiError::from)?;
        let parsed: CreateBatchResponse = parse_json(resp).await?;
        Ok(BatchId(parsed.id))
    }

    /// Poll a batch for its current processing status + per-request counts.
    pub async fn poll(&self, id: &BatchId) -> Result<BatchStatus, ApiError> {
        let url = format!("{}/{}", self.batches_url(), id.0);
        let resp = self
            .authed(self.http.get(&url))
            .send()
            .await
            .map_err(ApiError::from)?;
        let parsed: PollBatchResponse = parse_json(resp).await?;
        Ok(parsed.into_status(id.clone()))
    }

    /// Retrieve the JSONL results stream once the batch has ended. Each line is
    /// mapped to a [`BatchResultItem`] keyed by its `custom_id` (results are
    /// returned OUT OF SUBMISSION ORDER — callers MUST map by id, not position).
    pub async fn retrieve(&self, id: &BatchId) -> Result<Vec<BatchResultItem>, ApiError> {
        let url = format!("{}/{}/results", self.batches_url(), id.0);
        let resp = self
            .authed(self.http.get(&url))
            .send()
            .await
            .map_err(ApiError::from)?;
        let resp = resp.error_for_status().map_err(ApiError::from)?;
        let text = resp.text().await.map_err(ApiError::from)?;
        let mut out = Vec::new();
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            let raw: ResultLine = serde_json::from_str(line)?;
            out.push(raw.into_item());
        }
        Ok(out)
    }

    /// Request cancellation of an in-progress batch.
    pub async fn cancel(&self, id: &BatchId) -> Result<(), ApiError> {
        let url = format!("{}/{}/cancel", self.batches_url(), id.0);
        let resp = self
            .authed(self.http.post(&url))
            .send()
            .await
            .map_err(ApiError::from)?;
        resp.error_for_status().map_err(ApiError::from)?;
        Ok(())
    }
}

async fn parse_json<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T, ApiError> {
    let resp = resp.error_for_status().map_err(ApiError::from)?;
    resp.json::<T>().await.map_err(ApiError::from)
}

#[derive(Deserialize)]
struct CreateBatchResponse {
    id: String,
}

#[derive(Deserialize)]
struct PollBatchResponse {
    processing_status: String,
    #[serde(default)]
    request_counts: RequestCounts,
}

#[derive(Deserialize, Default)]
struct RequestCounts {
    #[serde(default)]
    processing: u32,
    #[serde(default)]
    succeeded: u32,
    #[serde(default)]
    errored: u32,
    #[serde(default)]
    canceled: u32,
    #[serde(default)]
    expired: u32,
}

impl PollBatchResponse {
    fn into_status(self, id: BatchId) -> BatchStatus {
        let processing_status = match self.processing_status.as_str() {
            "ended" => BatchProcessingStatus::Ended,
            "canceling" => BatchProcessingStatus::Canceling,
            // "in_progress" and any unknown future status are treated as
            // still-processing (fail-safe: never report Ended without proof).
            _ => BatchProcessingStatus::InProgress,
        };
        BatchStatus {
            id,
            processing_status,
            processing: self.request_counts.processing,
            succeeded: self.request_counts.succeeded,
            errored: self.request_counts.errored,
            canceled: self.request_counts.canceled,
            expired: self.request_counts.expired,
        }
    }
}

#[derive(Deserialize)]
struct ResultLine {
    custom_id: String,
    result: ResultBody,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ResultBody {
    Succeeded { message: MessageResponse },
    Errored { error: Value },
    Canceled,
    Expired,
}

impl ResultLine {
    fn into_item(self) -> BatchResultItem {
        let outcome = match self.result {
            ResultBody::Succeeded { message } => BatchResultOutcome::Succeeded(Box::new(message)),
            ResultBody::Errored { error } => BatchResultOutcome::Errored(error),
            ResultBody::Canceled => BatchResultOutcome::Canceled,
            ResultBody::Expired => BatchResultOutcome::Expired,
        };
        BatchResultItem {
            custom_id: self.custom_id,
            outcome,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_price_floors_never_rounds_up() {
        assert_eq!(half_price_cents(750), 375);
        assert_eq!(half_price_cents(1), 0);
        assert_eq!(half_price_cents(0), 0);
    }

    #[test]
    fn version_is_ga_no_beta() {
        let c = AnthropicBatchClient::new("k");
        assert_eq!(c.anthropic_version(), "2023-06-01");
    }

    #[test]
    fn batches_url_is_canonical() {
        let c = AnthropicBatchClient::new("k").with_base_url("https://example.test/");
        assert_eq!(c.batches_url(), "https://example.test/v1/messages/batches");
    }

    #[test]
    fn poll_status_parses_terminal_and_defaults_unknown_to_in_progress() {
        let ended: PollBatchResponse =
            serde_json::from_str(r#"{"processing_status":"ended","request_counts":{"succeeded":2}}"#)
                .expect("parse");
        let s = ended.into_status(BatchId("b1".into()));
        assert!(s.is_ended());
        assert_eq!(s.succeeded, 2);

        let unknown: PollBatchResponse =
            serde_json::from_str(r#"{"processing_status":"some_future_state"}"#).expect("parse");
        assert_eq!(
            unknown.into_status(BatchId("b2".into())).processing_status,
            BatchProcessingStatus::InProgress
        );
    }

    #[test]
    fn result_line_maps_by_custom_id_out_of_order() {
        let line = r#"{"custom_id":"art-7","result":{"type":"canceled"}}"#;
        let parsed: ResultLine = serde_json::from_str(line).expect("parse");
        let item = parsed.into_item();
        assert_eq!(item.custom_id, "art-7");
        assert!(matches!(item.outcome, BatchResultOutcome::Canceled));
    }
}
