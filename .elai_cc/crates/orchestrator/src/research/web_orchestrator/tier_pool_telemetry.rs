//! E-48 Task 2 — tiered-search-pool telemetry adapter.
//!
//! The concrete `TieredSearchPool` + `BackendTier` types live in the
//! [`elai_web_search`] crate (see
//! `crates/web_search/src/tiered_search_pool.rs`). This module owns the
//! producer-side telemetry payload shape so the SSB row schema stays
//! anchored next to the other E-48 substrate modules.
//!
//! Naming clarification: the rename from `SourceTier` to `BackendTier`
//! (per the cross-check report's SPEC_DRIFT row) is implemented on the
//! web_search side. The existing
//! [`elai_core::source_tier::SourceTier`] enum
//! (T1Authoritative / T2Strong / T3Weak / T4Rejected) keeps its domain-
//! quality semantics; the new `BackendTier` enum carries
//! backend-identity semantics (T1 = Brave / T2 = SearXNG / T3 = Mojeek).
//!
//! Grounding:
//!   * arXiv:2509.23519 -- ReliabilityRAG (per-source reliability tiers
//!     -- grounds the tier-rationale design).
//!   * arXiv:2410.22954 -- source-reliability heterogeneity (grounds the
//!     tiered-fan-out vs naive-retrieval choice).
//!   * costbench.com 2026 / api-dashboard.search.brave.com/documentation/
//!     pricing -- Brave $5/1k after free credits (grounds the T1 cost
//!     model and the `tier_disabled_quota_exceeded` status enum value).
//!   * factually.co 2026 -- DuckDuckGo Bing-derived not independent
//!     (grounds the explicit no-DDG-as-tier rule -- see stub_ban in the
//!     plan WIRING_CONTRACT).

use serde::{Deserialize, Serialize};

/// Telemetry status carried in the `tiered_search_dispatched` SSB row.
///
/// `Ok` is the happy path; the two non-Ok variants surface degraded
/// behaviour without breaking the request. Folding both into a single
/// Kind avoids two SsbActionType variants for the same fire-point
/// (per the Pattern-2 single-variant precedent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TieredDispatchStatus {
    /// At least one tier returned results; dispatch successful.
    Ok,
    /// T1 + T3 keys absent OR quota exhausted; falling through to
    /// T2-only SearXNG path. Legacy B-7 behaviour preserved.
    DegradedToT2Only,
    /// A specific tier hit its rate-limit / free-quota ceiling and
    /// got disabled for the rest of the session. The
    /// `disabled_tier` field in `details_json` disambiguates which.
    TierDisabledQuotaExceeded,
}

impl TieredDispatchStatus {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::DegradedToT2Only => "degraded_to_t2_only",
            Self::TierDisabledQuotaExceeded => "tier_disabled_quota_exceeded",
        }
    }
}

/// Telemetry payload for one `tiered_search_dispatched` row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TieredDispatchTelemetry {
    /// blake3 prefix of the canonicalized query string (8 hex chars).
    pub query_hash_prefix: String,
    pub status: TieredDispatchStatus,
    pub tier_t1_results: u32,
    pub tier_t2_results: u32,
    pub tier_t3_results: u32,
    /// Number of duplicate-URL rows dropped by the canonical-hash dedup.
    pub dedup_dropped: u32,
    /// When `status == TierDisabledQuotaExceeded`, the tier-label
    /// (`"t1"` / `"t3"`) that hit its quota. `None` otherwise.
    pub disabled_tier: Option<String>,
}

/// Telemetry payload for one `extraction_failed_all_extractors` row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractionFailedTelemetry {
    /// blake3 prefix of the canonicalized URL (8 hex chars).
    pub url_hash_prefix: String,
    pub primary_error: String,
    pub fallback_error: String,
    pub primary_timeout_ms: u32,
    pub fallback_timeout_ms: u32,
}

/// Telemetry payload for one `cross_encoder_rerank_complete` row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RerankCompleteTelemetry {
    pub n_input: u32,
    pub n_output: u32,
    /// One of `"local_sidecar"` / `"cohere"` / `"none"` (when the
    /// reranker is unavailable and the top-N is returned unchanged).
    pub reranker_backend: String,
    /// Histogram of rank shifts. Index i = number of sources whose
    /// final position differed from their input position by exactly
    /// (i + 1) ranks. Length capped at 5 to mirror the top-5 output.
    pub rank_shift_histogram: [u32; 5],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiered_status_round_trips_serde() {
        for s in [
            TieredDispatchStatus::Ok,
            TieredDispatchStatus::DegradedToT2Only,
            TieredDispatchStatus::TierDisabledQuotaExceeded,
        ] {
            let bytes = serde_json::to_vec(&s).expect("encode");
            let back: TieredDispatchStatus = serde_json::from_slice(&bytes).expect("decode");
            assert_eq!(s, back);
        }
    }

    #[test]
    fn tiered_status_as_str_canonical() {
        assert_eq!(TieredDispatchStatus::Ok.as_str(), "ok");
        assert_eq!(
            TieredDispatchStatus::DegradedToT2Only.as_str(),
            "degraded_to_t2_only"
        );
        assert_eq!(
            TieredDispatchStatus::TierDisabledQuotaExceeded.as_str(),
            "tier_disabled_quota_exceeded"
        );
    }

    #[test]
    fn rerank_telemetry_round_trips() {
        let t = RerankCompleteTelemetry {
            n_input: 50,
            n_output: 5,
            reranker_backend: "local_sidecar".to_owned(),
            rank_shift_histogram: [3, 2, 0, 0, 0],
        };
        let bytes = serde_json::to_vec(&t).expect("encode");
        let back: RerankCompleteTelemetry =
            serde_json::from_slice(&bytes).expect("decode");
        assert_eq!(t, back);
    }
}
