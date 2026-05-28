//! Producer-side SSB emit fns for the planning-decomposition substrate.
//!
//! Each `emit_*` free async fn:
//!   1. Checks [`super::is_enabled`] — silently returns `Ok(None)` if the
//!      env gate [`super::PLANNING_DECOMPOSITION_ENABLE_ENV`] is not set
//!      to a truthy value (wire-dark default).
//!   2. Builds a [`crate::ssb::PlanningDecompositionEventPayload`] with the
//!      provided Kind discriminator + free-form details_json.
//!   3. Writes one SSB row via the supplied `&Ssb` handle's async
//!      `write_payload` method.
//!
//! Single-writer producer-pure split per the dual-ledger + goal-drift
//! convention (modules under [`super`] take no `&Ssb`; producers live
//! here). 8 fire-points covered by the single
//! `PlanningDecompositionEvent` variant per the Pattern-2 single-variant +
//! N-arm Kind discriminator precedent (E-20 / E-21 / E-36 / E-42).
//!
//! All `emit_*` fns return `Result<Option<i64>, SsbError>`:
//!   * `Ok(None)` — env gate off, no row written, no error.
//!   * `Ok(Some(seq))` — row written; `seq` is the assigned SSB
//!     auto-increment sequence number.
//!   * `Err(SsbError)` — propagate the underlying write failure (incl
//!     `SsbError::PayloadValidation` for the stub_ban §88 predicates).

use serde_json::json;

use crate::ssb::{
    PlanningDecompositionEventKind, PlanningDecompositionEventPayload, Ssb, SsbError, SsbPayload,
};

use super::audit_score::ProcessRewardScore;
use super::decomposition_entropy::EntropyVerdict;
use super::goal_drift_ext::GoalDriftReport;
use super::reflection_buffer::Reflection;
use super::skill_prior::SkillPriorMatch;

/// Emit a `reflection_appended` row.
pub async fn emit_reflection_appended(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    node_key: &str,
    reflection: &Reflection,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !super::is_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::PlanningDecompositionEvent(PlanningDecompositionEventPayload {
        kind: PlanningDecompositionEventKind::ReflectionAppended,
        session_id: session_id.map(str::to_owned),
        task_id: Some(node_key.to_owned()),
        details_json: json!({
            "node_key": node_key,
            "trust_tier": reflection.trust_tier.as_str(),
            "source_event_id": reflection.source_event_id,
            "text_len": reflection.text.len(),
            "created_at_unix": reflection.created_at_unix,
        }),
        ts_ms,
    });
    let seq = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(Some(seq))
}

/// Emit a `low_entropy_decomposition` row. Only writes when the verdict is
/// [`EntropyVerdict::LowEntropy`] — `Ok` / `Skipped` verdicts are no-ops.
pub async fn emit_low_entropy_decomposition(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    parent_node_key: &str,
    verdict: &EntropyVerdict,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !super::is_enabled() {
        return Ok(None);
    }
    let (ratio, h_parent, h_children, threshold) = match verdict {
        EntropyVerdict::LowEntropy {
            ratio,
            h_parent,
            h_children,
            threshold,
        } => (*ratio, *h_parent, *h_children, *threshold),
        _ => return Ok(None),
    };
    let payload = SsbPayload::PlanningDecompositionEvent(PlanningDecompositionEventPayload {
        kind: PlanningDecompositionEventKind::LowEntropyDecomposition,
        session_id: session_id.map(str::to_owned),
        task_id: Some(parent_node_key.to_owned()),
        details_json: json!({
            "parent_node_key": parent_node_key,
            "entropy_ratio": ratio,
            "h_parent": h_parent,
            "h_children": h_children,
            "threshold": threshold,
            "verdict": verdict.as_str(),
        }),
        ts_ms,
    });
    let seq = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(Some(seq))
}

/// Emit a `goal_drift_canary_below_threshold` row. Plan stub_ban §89: BOTH
/// jaccard AND cosine must be populated; [`GoalDriftReport`] enforces this
/// at the type level. Only writes when `report.below_threshold == true`.
pub async fn emit_goal_drift_canary_below_threshold(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    phase_node_key: &str,
    report: &GoalDriftReport,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !super::is_enabled() {
        return Ok(None);
    }
    if !report.below_threshold {
        return Ok(None);
    }
    let payload = SsbPayload::PlanningDecompositionEvent(PlanningDecompositionEventPayload {
        kind: PlanningDecompositionEventKind::GoalDriftCanaryBelowThreshold,
        session_id: session_id.map(str::to_owned),
        task_id: Some(phase_node_key.to_owned()),
        details_json: json!({
            "phase_node_key": phase_node_key,
            "jaccard": report.jaccard,
            "cosine": report.cosine,
            "prev_below_threshold": report.prev_below_threshold,
            "fire_cascade": report.fire_cascade,
        }),
        ts_ms,
    });
    let seq = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(Some(seq))
}

/// Emit a `bubble_up_replan_phase` row. Stub_ban §88: callers MUST supply
/// non-empty `failed_plan`, `error_transcript`, `original_goal` — this fn
/// returns `SsbError::PayloadValidation` for any empty field.
pub async fn emit_bubble_up_replan_phase(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    phase_node_key: &str,
    failed_plan: &str,
    error_transcript: &str,
    original_goal: &str,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !super::is_enabled() {
        return Ok(None);
    }
    if failed_plan.trim().is_empty() {
        return Err(SsbError::PayloadValidation(
            "bubble_up_replan_phase: failed_plan must be non-empty (stub_ban §88)".to_owned(),
        ));
    }
    if error_transcript.trim().is_empty() {
        return Err(SsbError::PayloadValidation(
            "bubble_up_replan_phase: error_transcript must be non-empty (stub_ban §88)"
                .to_owned(),
        ));
    }
    if original_goal.trim().is_empty() {
        return Err(SsbError::PayloadValidation(
            "bubble_up_replan_phase: original_goal must be non-empty (stub_ban §88)".to_owned(),
        ));
    }
    let payload = SsbPayload::PlanningDecompositionEvent(PlanningDecompositionEventPayload {
        kind: PlanningDecompositionEventKind::BubbleUpReplanPhase,
        session_id: session_id.map(str::to_owned),
        task_id: Some(phase_node_key.to_owned()),
        details_json: json!({
            "phase_node_key": phase_node_key,
            "failed_plan": failed_plan,
            "error_transcript": error_transcript,
            "original_goal": original_goal,
        }),
        ts_ms,
    });
    let seq = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(Some(seq))
}

/// Emit a `bubble_up_ledger_reset` row. Stub_ban §88: caller MUST supply
/// non-empty `concept_or_phase_id`.
pub async fn emit_bubble_up_ledger_reset(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    concept_or_phase_id: &str,
    blocker_count: usize,
    threshold: usize,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !super::is_enabled() {
        return Ok(None);
    }
    if concept_or_phase_id.trim().is_empty() {
        return Err(SsbError::PayloadValidation(
            "bubble_up_ledger_reset: concept_or_phase_id must be non-empty (stub_ban §88)"
                .to_owned(),
        ));
    }
    let payload = SsbPayload::PlanningDecompositionEvent(PlanningDecompositionEventPayload {
        kind: PlanningDecompositionEventKind::BubbleUpLedgerReset,
        session_id: session_id.map(str::to_owned),
        task_id: Some(concept_or_phase_id.to_owned()),
        details_json: json!({
            "concept_or_phase_id": concept_or_phase_id,
            "blocker_count": blocker_count,
            "threshold": threshold,
        }),
        ts_ms,
    });
    let seq = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(Some(seq))
}

/// Emit an `agentless_shortcut_dispatched` row. Stub_ban §90 (advisory):
/// caller SHOULD verify impact-mapper has run in the same session before
/// dispatch.
pub async fn emit_agentless_shortcut_dispatched(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    step_node_key: &str,
    reason: &str,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !super::is_enabled() {
        return Ok(None);
    }
    if step_node_key.trim().is_empty() {
        return Err(SsbError::PayloadValidation(
            "agentless_shortcut_dispatched: step_node_key must be non-empty".to_owned(),
        ));
    }
    let payload = SsbPayload::PlanningDecompositionEvent(PlanningDecompositionEventPayload {
        kind: PlanningDecompositionEventKind::AgentlessShortcutDispatched,
        session_id: session_id.map(str::to_owned),
        task_id: Some(step_node_key.to_owned()),
        details_json: json!({
            "step_node_key": step_node_key,
            "reason": reason,
        }),
        ts_ms,
    });
    let seq = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(Some(seq))
}

/// Emit a `process_reward_score_emitted` row.
pub async fn emit_process_reward_score_emitted(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    tier_node_key: &str,
    score: &ProcessRewardScore,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !super::is_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::PlanningDecompositionEvent(PlanningDecompositionEventPayload {
        kind: PlanningDecompositionEventKind::ProcessRewardScoreEmitted,
        session_id: session_id.map(str::to_owned),
        task_id: Some(tier_node_key.to_owned()),
        details_json: json!({
            "tier_node_key": tier_node_key,
            "score": score.score,
            "verdict_modifier_label": score.verdict_modifier_label.as_str(),
            "findings_count": score.findings_count,
            "max_expected_findings": score.max_expected_findings,
        }),
        ts_ms,
    });
    let seq = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(Some(seq))
}

/// Emit a `skill_prior_retrieved` row for one match.
pub async fn emit_skill_prior_retrieved(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    query_node_key: &str,
    match_: &SkillPriorMatch,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !super::is_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::PlanningDecompositionEvent(PlanningDecompositionEventPayload {
        kind: PlanningDecompositionEventKind::SkillPriorRetrieved,
        session_id: session_id.map(str::to_owned),
        task_id: Some(query_node_key.to_owned()),
        details_json: json!({
            "query_node_key": query_node_key,
            "prior_id": match_.prior_id,
            "similarity": match_.similarity,
            "ssb_evidence_seqs": match_.ssb_evidence_seqs,
            "accepted_diff_summary_len": match_.accepted_diff_summary.len(),
        }),
        ts_ms,
    });
    let seq = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(Some(seq))
}
