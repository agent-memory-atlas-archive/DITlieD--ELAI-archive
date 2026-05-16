// B-41 Slice 2d — Per-Level Audit Dispatcher
//
// Glue layer between `auditor_mode` (Slice 1 types + persistence + respawn
// guard) and the live SSB / pipeline tree navigator. Three responsibilities:
//
//   1. Emit `per_level_audit_started` rows immediately before an Auditor*
//      agent is spawned at a tier node.
//   2. Emit `per_level_audit_verdict` rows after the verdict is produced.
//   3. On a 2nd BLOCK at the same node (`respawned_once == true` and the
//      fresh verdict is still BLOCK) — mark the verdict for cascade, emit
//      a `per_level_audit_cascade` row, and return the cascade target tier
//      label so the navigator can call its existing
//      `emit_integration_bubble_up` path for the parent decomposer.
//
// The dispatcher is split out from `pipeline_tree_navigator` so:
//   a) the navigator (2800+ LOC) stays untouched at the structural level,
//   b) the cascade policy (BabyStepDetail -> BabyStep, see
//      `B-41-decisions.md DECISION-CASCADE`) lives in one auditable place,
//   c) unit tests cover the dispatcher pure-functionally without spinning
//      up a real ConceptDecomposition fixture.
//
// Author contract per task brief: "ACTUALLY CALL the 5 derive_X_zero_blocks
// helpers at Evidence construction site — don't repeat B-40's dead-code bug."
// The dispatcher emits the SSB rows that `task_runner::derive_*_zero_blocks`
// then read back — wiring the producer→consumer pair end-to-end.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::auditor_mode::{
    mark_cascade, mark_respawned, AuditorMode, AuditorModeError, LevelAuditVerdict, TierNodeId,
    Verdict,
};
use crate::ssb::{
    PerLevelAuditCascadePayload, PerLevelAuditStartedPayload, PerLevelAuditVerdictPayload, Ssb,
    SsbPayload,
};

// ---------------------------------------------------------------------------
// CascadeOutcome
// ---------------------------------------------------------------------------

/// Outcome of a `record_verdict` call. The navigator pattern-matches on this
/// to decide whether to respawn the same-tier decomposer or bubble up.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CascadeOutcome {
    /// Allow verdict — proceed to the next tier.
    Pass,
    /// First BLOCK at this node — respawn the same-tier decomposer ONCE
    /// with the verdict findings as a rescope hint. `respawned_once` is now
    /// true on the persisted verdict, so a 2nd BLOCK forces `Cascade`.
    RespawnOnce { verdict: LevelAuditVerdict },
    /// Second BLOCK at this node — cascade to parent. `parent_tier` is the
    /// tier label of the immediate parent in the decomposition hierarchy.
    /// For BabyStepDetail this is "baby_step" (per
    /// `B-41-decisions.md DECISION-CASCADE`).
    Cascade {
        verdict: LevelAuditVerdict,
        parent_tier: &'static str,
    },
    /// Second BLOCK at the TOP tier (Concepts mode at a Concept tier node).
    /// The decomposition root has no parent decomposer to cascade INTO —
    /// `parent_tier_label` would self-loop to "concept" and the navigator
    /// would respawn the Designer infinitely. This outcome signals the
    /// navigator to escalate to the FRAMING substate (B-40) so the idea
    /// itself is re-framed before retrying decomposition. The SSB cascade
    /// row carries `parent_tier="idea"` as a distinguishing marker so the
    /// dashboard / navigator can route to FRAMING retry instead of looping.
    EscalateToIdea {
        verdict: LevelAuditVerdict,
    },
}

/// Map a tier node id to its cascade parent tier label. Pure function —
/// extracted so the unit tests can lock the parent mapping without a real
/// navigator fixture.
///
/// Mapping (per `B-41-decisions.md`):
///   - concept       -> concept   (top tier self-loop; `record_verdict`
///                                   detects this pair and routes to
///                                   `CascadeOutcome::EscalateToIdea` so the
///                                   navigator escalates to the FRAMING
///                                   substate (B-40) instead of respawning
///                                   the Designer infinitely)
///   - phase         -> concept
///   - step          -> phase
///   - baby_step     -> step
///   - babystep_detail audit cascades to baby_step (NOT a separate
///     DecompositionLevel; see DECISION-CASCADE)
pub fn parent_tier_label(mode: AuditorMode, tier: TierNodeId) -> &'static str {
    match (mode, tier) {
        // BabyStepDetail collapses INTO the BabyStep tier on cascade.
        (AuditorMode::BabyStepDetail, _) => "baby_step",
        (_, TierNodeId::Concept(_)) => "concept",
        (_, TierNodeId::Phase(_)) => "concept",
        (_, TierNodeId::Step(_)) => "phase",
        (_, TierNodeId::BabyStep(_)) => "step",
    }
}

// ---------------------------------------------------------------------------
// Started-row emission
// ---------------------------------------------------------------------------

/// Emit a `per_level_audit_started` SSB row. Returns the seq on success.
/// On SSB write failure logs at WARN and returns `Err` — callers should
/// treat the audit as best-effort observable rather than abort.
///
/// Caller convention: invoke this IMMEDIATELY before the Auditor* agent
/// spawn so dashboard timeline rows align with agent lifecycle rows.
pub async fn emit_started(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    mode: AuditorMode,
    tier: TierNodeId,
) -> Result<i64, AuditorModeError> {
    let payload = SsbPayload::PerLevelAuditStarted(PerLevelAuditStartedPayload {
        mode: mode.as_str().to_owned(),
        tier: tier.tier_str().to_owned(),
        node_id: tier.raw_id(),
    });
    writer
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await
        .map_err(|e| AuditorModeError::Persistence(format!("per_level_audit_started: {e}")))
}

// ---------------------------------------------------------------------------
// Verdict-row emission + cascade decision
// ---------------------------------------------------------------------------

/// Emit a `per_level_audit_verdict` SSB row for the current verdict (no
/// respawn/cascade logic — pure recording).
async fn emit_verdict_row(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    v: &LevelAuditVerdict,
) -> Result<i64, AuditorModeError> {
    let payload = SsbPayload::PerLevelAuditVerdict(PerLevelAuditVerdictPayload {
        mode: v.mode.as_str().to_owned(),
        tier: v.tier_node_id.tier_str().to_owned(),
        node_id: v.tier_node_id.raw_id(),
        verdict: match v.verdict {
            Verdict::Allow => "allow".to_owned(),
            Verdict::Block => "block".to_owned(),
        },
        // NARROWING-OK: findings_count is bounded by the auditor agent's
        // finding cap (≤ 1024 per spec); u32 has 4-billion headroom.
        findings_count: u32::try_from(v.findings.len()).unwrap_or(u32::MAX),
        respawned_once: v.respawned_once,
        cascade_to_parent: v.cascade_to_parent,
    });
    writer
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await
        .map_err(|e| AuditorModeError::Persistence(format!("per_level_audit_verdict: {e}")))
}

/// Emit a `per_level_audit_cascade` SSB row when a verdict has been marked
/// `cascade_to_parent=true`. The `parent_tier` label is supplied by
/// `parent_tier_label` and recorded so the dashboard can render the arrow
/// without re-running the cascade policy.
async fn emit_cascade_row(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    v: &LevelAuditVerdict,
    parent_tier: &'static str,
) -> Result<i64, AuditorModeError> {
    let payload = SsbPayload::PerLevelAuditCascade(PerLevelAuditCascadePayload {
        mode: v.mode.as_str().to_owned(),
        tier: v.tier_node_id.tier_str().to_owned(),
        node_id: v.tier_node_id.raw_id(),
        parent_tier: parent_tier.to_owned(),
    });
    writer
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await
        .map_err(|e| AuditorModeError::Persistence(format!("per_level_audit_cascade: {e}")))
}

/// Record a verdict produced by an Auditor* agent and decide the next move.
///
/// Pipeline (in order):
///   1. Emit `per_level_audit_verdict` SSB row for `verdict` as-given.
///   2. If `verdict.verdict == Allow` -> return `Pass`.
///   3. Else if `verdict.respawned_once == false`:
///        - `mark_respawned(verdict)` -> updated verdict with respawned=true.
///        - Emit a SECOND verdict row reflecting respawned=true (so timeline
///          shows the respawn was committed even if the same-tier rerun
///          never lands a follow-up).
///        - Return `RespawnOnce { verdict: updated }`.
///   4. Else (`respawned_once == true` AND verdict is still Block):
///        - `mark_cascade(verdict)` -> cascade_to_parent=true.
///        - Emit a `per_level_audit_cascade` row.
///        - Return `Cascade { verdict: cascaded, parent_tier }`.
///
/// The verdict markdown is persisted to disk separately via
/// `auditor_mode::persist_verdict`; this function only writes the SSB rows
/// so the dashboard timeline and the `derive_*_zero_blocks` evidence
/// helpers can observe the cascade in real time.
pub async fn record_verdict(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    verdict: LevelAuditVerdict,
) -> Result<CascadeOutcome, AuditorModeError> {
    // Step 1 — record verdict as-given.
    emit_verdict_row(writer, agent_id, superstep, &verdict).await?;

    // Step 2 — Allow path.
    if verdict.verdict == Verdict::Allow {
        return Ok(CascadeOutcome::Pass);
    }

    // Step 3 — first BLOCK -> respawn once.
    if !verdict.respawned_once {
        let updated = mark_respawned(verdict)?;
        // Second row so timeline reflects respawned=true even if the
        // retry never produces a follow-up verdict.
        emit_verdict_row(writer, agent_id, superstep, &updated).await?;
        return Ok(CascadeOutcome::RespawnOnce { verdict: updated });
    }

    // Step 4 — second BLOCK -> cascade.
    let parent = parent_tier_label(verdict.mode, verdict.tier_node_id);
    let current_tier = verdict.tier_node_id.tier_str();
    // Self-loop detection: at the decomposition root (Concepts mode on a
    // Concept tier node) the parent label equals the current tier label.
    // Cascading to "concept" would respawn the same-tier Designer forever,
    // so escalate to FRAMING instead. The cascade SSB row uses
    // parent_tier="idea" as a distinguishing marker so the dashboard and
    // navigator route to the FRAMING retry path (B-40) rather than the
    // standard parent-decomposer respawn path.
    // Only the Concepts-mode-at-Concept-tier pair is a TRUE self-loop.
    // BabyStepDetail collapsing to "baby_step" is an INTENTIONAL collapse
    // (DECISION-CASCADE) — the parent BabyStep decomposer still exists to
    // receive the bubble-up. The Concepts top tier has no parent
    // decomposer, so its cascade must be re-routed to FRAMING.
    let is_top_tier_self_loop = matches!(verdict.mode, AuditorMode::Concepts)
        && matches!(verdict.tier_node_id, TierNodeId::Concept(_))
        && parent == current_tier;
    let routed_parent: &'static str = if is_top_tier_self_loop {
        "idea"
    } else {
        parent
    };
    let cascaded = mark_cascade(verdict);
    // Re-emit the now-cascaded verdict so the
    // `derive_*_zero_blocks` evidence helpers can observe
    // cascade_to_parent=true on a fresh row.
    emit_verdict_row(writer, agent_id, superstep, &cascaded).await?;
    if let Err(e) = emit_cascade_row(writer, agent_id, superstep, &cascaded, routed_parent).await {
        warn!(
            target: "per_level_audit_dispatch",
            error = %e,
            mode = cascaded.mode.as_str(),
            tier = cascaded.tier_node_id.tier_str(),
            node_id = cascaded.tier_node_id.raw_id(),
            "cascade row emit failed; cascade decision still surfaced via verdict row"
        );
    }
    if is_top_tier_self_loop {
        Ok(CascadeOutcome::EscalateToIdea { verdict: cascaded })
    } else {
        Ok(CascadeOutcome::Cascade {
            verdict: cascaded,
            parent_tier: routed_parent,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::research_tree::{BabyStepId, ConceptId, PhaseId, StepId};

    #[test]
    fn parent_tier_concept_self_loops() {
        let t = TierNodeId::Concept(ConceptId(1));
        assert_eq!(parent_tier_label(AuditorMode::Concepts, t), "concept");
    }

    #[test]
    fn parent_tier_phase_to_concept() {
        let t = TierNodeId::Phase(PhaseId(1));
        assert_eq!(parent_tier_label(AuditorMode::Phases, t), "concept");
    }

    #[test]
    fn parent_tier_step_to_phase() {
        let t = TierNodeId::Step(StepId(1));
        assert_eq!(parent_tier_label(AuditorMode::Steps, t), "phase");
    }

    #[test]
    fn parent_tier_baby_step_to_step() {
        let t = TierNodeId::BabyStep(BabyStepId(1));
        assert_eq!(parent_tier_label(AuditorMode::BabySteps, t), "step");
    }

    #[test]
    fn parent_tier_babystep_detail_to_baby_step_regardless_of_inner_id() {
        // DECISION-CASCADE: BabyStepDetail collapses into the BabyStep tier
        // EVEN IF the inner TierNodeId variant says otherwise (which it
        // should not by construction, but the policy is robust to that).
        for tier in [
            TierNodeId::Concept(ConceptId(1)),
            TierNodeId::Phase(PhaseId(1)),
            TierNodeId::Step(StepId(1)),
            TierNodeId::BabyStep(BabyStepId(1)),
        ] {
            assert_eq!(
                parent_tier_label(AuditorMode::BabyStepDetail, tier),
                "baby_step",
                "BabyStepDetail must always cascade to baby_step (tier={:?})",
                tier
            );
        }
    }
}
