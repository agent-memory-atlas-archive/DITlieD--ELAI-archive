//! Producer-side SSB emit fns for the E-48 web-research orchestrator
//! hardening substrate.
//!
//! Each `emit_*` free async fn:
//!   1. Checks [`super::is_enabled`] -- silently returns `Ok(None)` if
//!      the env gate [`super::WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV`] is
//!      not set to a truthy value (wire-dark default).
//!   2. Builds a [`crate::ssb::WebResearchEventPayload`] with the
//!      provided Kind discriminator + free-form `details_json`.
//!   3. Writes one SSB row via the supplied `&Ssb` handle's async
//!      `write_payload` method.
//!
//! Single-writer producer-pure split mirrors the
//! [`crate::planning_decomposition::producers`] convention. 10 fire-points
//! covered by the single [`crate::ssb::SsbActionType::WebResearchEvent`]
//! variant per the Pattern-2 N-arm Kind discriminator precedent.
//!
//! All `emit_*` fns return `Result<Option<i64>, SsbError>`:
//!   * `Ok(None)` -- env gate off, no row written, no error.
//!   * `Ok(Some(seq))` -- row written; `seq` is the assigned SSB
//!     auto-increment sequence number.
//!   * `Err(SsbError)` -- propagate the underlying write failure.

use serde_json::json;

use crate::ssb::{
    Ssb, SsbError, SsbPayload, WebResearchEventKind, WebResearchEventPayload,
};

use super::citation_verifier::{
    AttributionLookup, AttributionSet, VerificationOutcome,
};
use super::contradiction_classifier::ScanOutcome;
use super::editable_plan::WebResearchPlan;
use super::recency_decay;
use super::speculative_pruner::PrunerVerdict;
use super::tier_pool_telemetry::{
    ExtractionFailedTelemetry, RerankCompleteTelemetry, TieredDispatchTelemetry,
};

// ---------------------------------------------------------------------------
// Task 1 — research_plan_emitted
// ---------------------------------------------------------------------------

/// Emit a `research_plan_emitted` row.
pub async fn emit_research_plan_emitted(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    node_id: Option<&str>,
    plan: &WebResearchPlan,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !super::is_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::WebResearchEvent(WebResearchEventPayload {
        kind: WebResearchEventKind::ResearchPlanEmitted,
        session_id: session_id.map(str::to_owned),
        node_id: node_id.map(str::to_owned),
        details_json: json!({
            "sub_question_count": plan.sub_question_count(),
            "planned_search_budget": plan.planned_search_budget,
            "interactive": plan.interactive,
            "approved_by_user": plan.approved_by_user,
            "parent_question_len": plan.parent_question.len(),
        }),
        ts_ms,
    });
    let seq = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(Some(seq))
}

// ---------------------------------------------------------------------------
// Task 2 — tiered_search_dispatched
// ---------------------------------------------------------------------------

/// Emit a `tiered_search_dispatched` row.
pub async fn emit_tiered_search_dispatched(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    node_id: Option<&str>,
    telemetry: &TieredDispatchTelemetry,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !super::is_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::WebResearchEvent(WebResearchEventPayload {
        kind: WebResearchEventKind::TieredSearchDispatched,
        session_id: session_id.map(str::to_owned),
        node_id: node_id.map(str::to_owned),
        details_json: json!({
            "query_hash_prefix": telemetry.query_hash_prefix,
            "status": telemetry.status.as_str(),
            "tier_t1_results": telemetry.tier_t1_results,
            "tier_t2_results": telemetry.tier_t2_results,
            "tier_t3_results": telemetry.tier_t3_results,
            "dedup_dropped": telemetry.dedup_dropped,
            "disabled_tier": telemetry.disabled_tier,
        }),
        ts_ms,
    });
    let seq = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(Some(seq))
}

// ---------------------------------------------------------------------------
// Task 3 — extraction_failed_all_extractors
// ---------------------------------------------------------------------------

/// Emit an `extraction_failed_all_extractors` row.
pub async fn emit_extraction_failed_all_extractors(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    node_id: Option<&str>,
    telemetry: &ExtractionFailedTelemetry,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !super::is_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::WebResearchEvent(WebResearchEventPayload {
        kind: WebResearchEventKind::ExtractionFailedAllExtractors,
        session_id: session_id.map(str::to_owned),
        node_id: node_id.map(str::to_owned),
        details_json: json!({
            "url_hash_prefix": telemetry.url_hash_prefix,
            "primary_error": telemetry.primary_error,
            "fallback_error": telemetry.fallback_error,
            "primary_timeout_ms": telemetry.primary_timeout_ms,
            "fallback_timeout_ms": telemetry.fallback_timeout_ms,
        }),
        ts_ms,
    });
    let seq = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(Some(seq))
}

// ---------------------------------------------------------------------------
// Task 4 — cross_encoder_rerank_complete
// ---------------------------------------------------------------------------

/// Emit a `cross_encoder_rerank_complete` row.
pub async fn emit_cross_encoder_rerank_complete(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    node_id: Option<&str>,
    telemetry: &RerankCompleteTelemetry,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !super::is_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::WebResearchEvent(WebResearchEventPayload {
        kind: WebResearchEventKind::CrossEncoderRerankComplete,
        session_id: session_id.map(str::to_owned),
        node_id: node_id.map(str::to_owned),
        details_json: json!({
            "n_input": telemetry.n_input,
            "n_output": telemetry.n_output,
            "reranker_backend": telemetry.reranker_backend,
            "rank_shift_histogram": telemetry.rank_shift_histogram,
        }),
        ts_ms,
    });
    let seq = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(Some(seq))
}

// ---------------------------------------------------------------------------
// Task 5 — speculative_branch_pruned
// ---------------------------------------------------------------------------

/// Emit a `speculative_branch_pruned` row. Only emits when
/// `verdict.should_prune == true`; non-prune verdicts short-circuit.
pub async fn emit_speculative_branch_pruned(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    branch_id: u64,
    verdict: &PrunerVerdict,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !super::is_enabled() {
        return Ok(None);
    }
    if !verdict.should_prune {
        return Ok(None);
    }
    let payload = SsbPayload::WebResearchEvent(WebResearchEventPayload {
        kind: WebResearchEventKind::SpeculativeBranchPruned,
        session_id: session_id.map(str::to_owned),
        node_id: Some(branch_id.to_string()),
        details_json: json!({
            "branch_id": branch_id,
            "branch_type": verdict.branch_type.as_str(),
            "evidence_delta": verdict.evidence_delta,
            "low_gain_streak": verdict.low_gain_streak,
            "threshold": verdict.threshold,
        }),
        ts_ms,
    });
    let seq = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(Some(seq))
}

// ---------------------------------------------------------------------------
// Task 6 — research_preemptive_stop
// ---------------------------------------------------------------------------

/// Emit a `research_preemptive_stop` row. Caller passes the
/// `PreemptiveUnsolvable` arm fields directly; the helper folds them
/// into the canonical `details_json` shape.
pub async fn emit_research_preemptive_stop(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    node_id: Option<&str>,
    rounds: u32,
    low_delta_streak: u32,
    final_confidence: f64,
    threshold_uncalibrated: bool,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !super::is_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::WebResearchEvent(WebResearchEventPayload {
        kind: WebResearchEventKind::ResearchPreemptiveStop,
        session_id: session_id.map(str::to_owned),
        node_id: node_id.map(str::to_owned),
        details_json: json!({
            "final_confidence": final_confidence,
            "rounds": rounds,
            "low_delta_streak": low_delta_streak,
            "threshold_uncalibrated": threshold_uncalibrated,
        }),
        ts_ms,
    });
    let seq = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(Some(seq))
}

// ---------------------------------------------------------------------------
// Task 7 — contradiction_node_materialized
// ---------------------------------------------------------------------------

/// Emit one row per Disagree contradiction in `outcome.contradictions`.
/// Returns the count of rows actually written (sum of `Some(_)` arms).
pub async fn emit_contradiction_node_materialized(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    node_id: Option<&str>,
    outcome: &ScanOutcome,
    ts_ms: i64,
) -> Result<usize, SsbError> {
    if !super::is_enabled() {
        return Ok(0);
    }
    let mut written = 0;
    for c in &outcome.contradictions {
        let payload = SsbPayload::WebResearchEvent(WebResearchEventPayload {
            kind: WebResearchEventKind::ContradictionNodeMaterialized,
            session_id: session_id.map(str::to_owned),
            node_id: node_id.map(str::to_owned),
            details_json: json!({
                "source_a_url_hash": blake3_prefix_hex(&c.source_a.url),
                "source_b_url_hash": blake3_prefix_hex(&c.source_b.url),
                "classification_reason": c.classification_reason.clone(),
                "prune_protected": c.prune_protected,
                "budget_clamped": outcome.budget_clamped,
                "surfaced_at_unix_ms": c.surfaced_at_unix_ms,
            }),
            ts_ms,
        });
        ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
        written += 1;
    }
    Ok(written)
}

// ---------------------------------------------------------------------------
// Task 8 — citation_verification_outcome
// ---------------------------------------------------------------------------

/// Emit a `citation_verification_outcome` row.
pub async fn emit_citation_verification_outcome(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    node_id: Option<&str>,
    citation_url: &str,
    outcome: &VerificationOutcome,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !super::is_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::WebResearchEvent(WebResearchEventPayload {
        kind: WebResearchEventKind::CitationVerificationOutcome,
        session_id: session_id.map(str::to_owned),
        node_id: node_id.map(str::to_owned),
        details_json: json!({
            "citation_url_hash": blake3_prefix_hex(citation_url),
            "outcome": outcome.ssb_outcome_label(),
            "url_live": outcome.url_live,
            "arxiv_valid": outcome.arxiv_valid,
            "doi_valid": outcome.doi_valid,
            "faithful": outcome.faithful.map(|f| f.as_str()),
        }),
        ts_ms,
    });
    let seq = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(Some(seq))
}

// ---------------------------------------------------------------------------
// Task 9 — citation_attribution_violation_blocked
// ---------------------------------------------------------------------------

/// Emit a `citation_attribution_violation_blocked` row when an
/// attribution-set lookup returns `Blocked`. Returns `Ok(None)` for
/// `Allowed` lookups so callers can fold this into the normal cite path
/// without a conditional emit on the call site.
pub async fn emit_citation_attribution_violation_blocked(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    node_id: Option<&str>,
    attempted_url: &str,
    lookup: AttributionLookup,
    producer_set: &AttributionSet,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !super::is_enabled() {
        return Ok(None);
    }
    if matches!(lookup, AttributionLookup::Allowed) {
        return Ok(None);
    }
    let payload = SsbPayload::WebResearchEvent(WebResearchEventPayload {
        kind: WebResearchEventKind::CitationAttributionViolationBlocked,
        session_id: session_id.map(str::to_owned),
        node_id: node_id.map(str::to_owned),
        details_json: json!({
            "attempted_url_hash": blake3_prefix_hex(attempted_url),
            "producer_set_size": producer_set.len(),
        }),
        ts_ms,
    });
    let seq = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(Some(seq))
}

// ---------------------------------------------------------------------------
// Task 10 — recency_decay_applied
// ---------------------------------------------------------------------------

/// Emit a `recency_decay_applied` row. Only emits when at least one
/// source was decayed.
pub async fn emit_recency_decay_applied(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    node_id: Option<&str>,
    half_life_days: u32,
    n_sources_decayed: u32,
    mean_age_days: f32,
    temporal_marker: &str,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !super::is_enabled() {
        return Ok(None);
    }
    if n_sources_decayed == 0 {
        return Ok(None);
    }
    let half = if half_life_days == 0 {
        recency_decay::DEFAULT_HALF_LIFE_DAYS
    } else {
        half_life_days
    };
    let payload = SsbPayload::WebResearchEvent(WebResearchEventPayload {
        kind: WebResearchEventKind::RecencyDecayApplied,
        session_id: session_id.map(str::to_owned),
        node_id: node_id.map(str::to_owned),
        details_json: json!({
            "half_life_days": half,
            "n_sources_decayed": n_sources_decayed,
            "mean_age_days": mean_age_days,
            "temporal_marker": temporal_marker,
        }),
        ts_ms,
    });
    let seq = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(Some(seq))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// blake3-prefix helper. Returns the first 8 hex chars of
/// blake3(input). Used everywhere this module needs to log a URL
/// without leaking the full string (CTIM-Rover defense; mirrors the
/// E-43 SkillSystemEvent payload `prompt_hash_prefix` shape).
///
/// CLAUDE.md anti-stale: blake3, NEVER blake2 / sha2.
fn blake3_prefix_hex(input: &str) -> String {
    let hash = blake3::hash(input.as_bytes());
    let hex = hash.to_hex();
    hex.as_str().chars().take(8).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blake3_prefix_is_eight_hex_chars() {
        let p = blake3_prefix_hex("https://arxiv.org/abs/2510.05145");
        assert_eq!(p.len(), 8);
        assert!(p.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn blake3_prefix_is_deterministic() {
        let a = blake3_prefix_hex("https://a.example.com");
        let b = blake3_prefix_hex("https://a.example.com");
        assert_eq!(a, b);
    }

    #[test]
    fn blake3_prefix_differs_for_different_inputs() {
        let a = blake3_prefix_hex("https://a.example.com");
        let b = blake3_prefix_hex("https://b.example.com");
        assert_ne!(a, b);
    }
}
