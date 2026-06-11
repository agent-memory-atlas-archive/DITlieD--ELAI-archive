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

use elai_core::clock::{ClockProvider, OsClock};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::auditor_mode::{
    mark_cascade, mark_respawned, AuditorMode, AuditorModeError, LevelAuditVerdict, Severity,
    TierNodeId, Verdict,
};
use crate::mast::finding_to_mast_mode;
use crate::planning_decomposition::audit_score::{compute_score, MAX_EXPECTED_FINDINGS};
use crate::planning_decomposition::goal_drift_ext::GoalDriftEvaluator;
use crate::planning_decomposition::producers::{
    emit_goal_drift_canary_below_threshold, emit_process_reward_score_emitted,
};
use crate::runtime_config::FailureTrackerRuntimeConfig;
use crate::ssb::{
    AuditMastTaggedPayload, PerLevelAuditCascadePayload, PerLevelAuditStartedPayload,
    PerLevelAuditVerdictPayload, Ssb, SsbPayload,
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
// RUNPOD-COMPLETE: 2026-05-29 E-46 ADaPT iteration measurement — wire
// `emit_process_reward_score_emitted` into the per-level audit dispatch path
// so every verdict (initial, respawned, cascaded) carries a LATS-style
// process-reward scalar onto the planning_decomposition SSB stream. The
// producer is wire-dark by default (gated by ELAI_PLANNING_DECOMPOSITION_ENABLE)
// so this is additive zero-cost when the gate is off.
/// E-6 — attribute a BLOCK verdict to a MAST failure mode and emit an
/// `audit_mast_tagged` SSB row joining back to the originating
/// `per_level_audit_verdict` row via `verdict_seq`.
///
/// Wire-dark unless `ELAI_FAILURE_TRACKER_ENABLE=1`
/// ([`FailureTrackerRuntimeConfig::is_enabled`], DECISION-6). The
/// underlying verdict row is ALWAYS recorded by the caller; this helper
/// only adds the MAST-mode attribution row on top.
///
/// Attribution uses the deterministic `mast::finding_to_mast_mode`
/// predicate over the FIRST BLOCK-severity finding's `category`. If no
/// finding maps cleanly to a MAST mode (predicate returns `None`), no row
/// is emitted — the absence of a clean mapping is recorded as the absence
/// of a row rather than a guessed tag (per plan §Open questions / risks).
///
/// `agent_role` is derived from the verdict mode via
/// [`AuditorMode::map_to_agent_role`]. Emit failure is logged at WARN and
/// swallowed — the verdict + cascade decision are already surfaced by the
/// caller's verdict row, so a missing telemetry row must not abort the
/// audit pipeline.
async fn emit_mast_tagged_for_block(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    verdict: &LevelAuditVerdict,
    verdict_seq: i64,
) {
    if !FailureTrackerRuntimeConfig::is_enabled() {
        return;
    }
    if verdict.verdict != Verdict::Block {
        return;
    }
    // Attribute on the first BLOCK-severity finding that maps to a MAST
    // mode. WARN/INFO findings are advisory and do not drive the verdict,
    // so they are not used for attribution.
    let mast_mode = verdict
        .findings
        .iter()
        .filter(|f| f.severity == Severity::Block)
        .find_map(|f| finding_to_mast_mode(&f.category));
    let mast_mode = match mast_mode {
        Some(m) => m,
        None => return,
    };
    // AgentRole serialises to its snake_case wire tag (rename_all =
    // "snake_case" on the enum); serde_json::to_value yields a JSON string
    // like "auditor_babysteps". Fall back to the bare mode label if the
    // (infallible-in-practice) serialisation ever fails.
    let role = verdict.mode.map_to_agent_role();
    let agent_role = serde_json::to_value(role)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| verdict.mode.as_str().to_owned());
    let ts_ms = i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX);
    let payload = SsbPayload::AuditMastTagged(AuditMastTaggedPayload {
        verdict_seq,
        mast_mode: mast_mode.as_sql_str().to_owned(),
        confidence_band: "medium".to_owned(),
        agent_role,
        ts_ms,
    });
    if let Err(e) = writer
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await
    {
        warn!(
            target: "per_level_audit_dispatch",
            error = %e,
            mast_mode = mast_mode.as_sql_str(),
            verdict_seq,
            "audit_mast_tagged emit failed; verdict still recorded"
        );
    }
}

async fn emit_process_reward_for_verdict(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    verdict: &LevelAuditVerdict,
) {
    let score = compute_score(verdict, MAX_EXPECTED_FINDINGS);
    let tier_node_key = format!(
        "{}/{}/{}",
        verdict.mode.as_str(),
        verdict.tier_node_id.tier_str(),
        verdict.tier_node_id.raw_id()
    );
    let ts_ms = i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX);
    if let Err(e) = emit_process_reward_score_emitted(
        writer.as_ref(),
        agent_id,
        superstep,
        None,
        &tier_node_key,
        &score,
        ts_ms,
    )
    .await
    {
        warn!(
            target: "per_level_audit_dispatch",
            error = %e,
            tier_node_key = %tier_node_key,
            "process_reward_score emit failed; verdict still recorded"
        );
    }
}

// ---------------------------------------------------------------------------
// E-46: Goal-drift canary wiring
// ---------------------------------------------------------------------------

/// Evaluate goal drift for a completed Phase boundary and emit a
/// `PlanningDecompositionEvent / GoalDriftCanaryBelowThreshold` SSB row
/// when the phase-acceptance text has drifted below threshold relative to
/// the parent-idea text.
///
/// Wire-dark unless `ELAI_PLANNING_DECOMPOSITION_ENABLE=1`. The row is
/// written by the PRODUCTION emit fn; this function is the non-test
/// production call-site that satisfies R6 (producer has a production caller
/// outside the module and its tests).
///
/// Returns the `below_threshold` bool so the caller can thread it as the
/// `prev_below_threshold` value for the NEXT phase evaluation (the 2-of-2
/// consecutive trigger from plan task 4 §43).
///
/// On SSB write failure the error is logged at WARN and `false` is returned
/// so the audit pipeline continues — the goal-drift canary is observational
/// telemetry, not a hard gate.
///
/// Production call chain:
///   start_task_runner -> run_task_runner -> handle_task_entry -> drive_task
///   -> emit_level_transitions (phase_changed branch)
///   -> emit_goal_drift_for_verdict
///   -> GoalDriftEvaluator::evaluate
///   -> emit_goal_drift_canary_below_threshold  [off-agent artifact row]
pub async fn emit_goal_drift_for_verdict(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    phase_acceptance_text: &str,
    idea_text: &str,
    prev_below_threshold: bool,
    jaccard_threshold: f32,
    cosine_threshold: f32,
) -> bool {
    let evaluator = GoalDriftEvaluator::with_thresholds(
        idea_text,
        jaccard_threshold,
        cosine_threshold,
    );
    let report = evaluator.evaluate(phase_acceptance_text, prev_below_threshold);
    let ts_ms = i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX);
    if let Err(e) = emit_goal_drift_canary_below_threshold(
        writer.as_ref(),
        agent_id,
        superstep,
        Some(writer.session_id()),
        phase_acceptance_text,
        &report,
        ts_ms,
    )
    .await
    {
        warn!(
            target: "per_level_audit_dispatch",
            error = %e,
            jaccard = report.jaccard,
            cosine = report.cosine,
            below_threshold = report.below_threshold,
            "goal_drift_canary emit failed; audit pipeline continues"
        );
    }
    report.below_threshold
}

pub async fn record_verdict(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    verdict: LevelAuditVerdict,
) -> Result<CascadeOutcome, AuditorModeError> {
    // Step 1 — record verdict as-given. Capture the SSB seq so the E-6
    // MAST attribution row can join back to this verdict via verdict_seq.
    let verdict_seq = emit_verdict_row(writer, agent_id, superstep, &verdict).await?;
    emit_process_reward_for_verdict(writer, agent_id, superstep, &verdict).await;
    // E-6 — emit audit_mast_tagged on a BLOCK verdict (wire-dark unless
    // ELAI_FAILURE_TRACKER_ENABLE=1). No-op for Allow + for BLOCKs whose
    // finding categories do not map to any MAST mode.
    emit_mast_tagged_for_block(writer, agent_id, superstep, &verdict, verdict_seq).await;

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
        emit_process_reward_for_verdict(writer, agent_id, superstep, &updated).await;
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
    emit_process_reward_for_verdict(writer, agent_id, superstep, &cascaded).await;
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
    use crate::auditor_mode::Finding;
    use crate::research_tree::{BabyStepId, ConceptId, PhaseId, StepId};
    use crate::ssb::{SsbConfig, SsbReader};
    use crate::ssb::SsbActionType;
    use elai_core::clock::SeededClock;
    use tempfile::TempDir;

    fn test_clock() -> Arc<SeededClock> {
        Arc::new(SeededClock::new())
    }

    fn test_config(dir: &TempDir) -> SsbConfig {
        SsbConfig {
            db_path: dir.path().join("per_level_audit_ssb.db"),
            pool_size: Some(2),
            project_id: "test-proj".into(),
            session_id: "test-session".into(),
        }
    }

    fn block_verdict(node: u64) -> LevelAuditVerdict {
        // BLOCK verdict — first BLOCK finding category "step_repetition"
        // maps to MastMode::Fm13StepRepetition via finding_to_mast_mode.
        LevelAuditVerdict {
            mode: AuditorMode::BabySteps,
            tier_node_id: TierNodeId::BabyStep(BabyStepId(node)),
            findings: vec![
                Finding {
                    category: "step_repetition".into(),
                    severity: Severity::Block,
                    message: "agent repeated the same edit".into(),
                    source: None,
                },
                Finding {
                    category: "style_nit".into(),
                    severity: Severity::Warn,
                    message: "advisory".into(),
                    source: None,
                },
            ],
            verdict: Verdict::Block,
            respawned_once: false,
            cascade_to_parent: false,
        }
    }

    /// E-6 off-agent smoke (single sequential test exercising BOTH the
    /// gate-ON emit and the default-OFF suppression). It drives the REAL
    /// `record_verdict` dispatch hook with a BLOCK verdict whose finding
    /// category maps to a MAST mode, then reads back the `audit_mast_tagged`
    /// row the PRODUCTION code wrote (NOT authored by the test) via
    /// `Ssb::query_by_type` — the off-agent artifact. Merged into one test
    /// because both legs mutate the process-global `ELAI_FAILURE_TRACKER_ENABLE`
    /// var and a parallel split would race (per ENV NOTE).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn record_verdict_block_audit_mast_tagged_gate_on_and_off() {
        // ----- Leg 1: gate ON -> exactly one audit_mast_tagged row -----
        std::env::set_var("ELAI_FAILURE_TRACKER_ENABLE", "1");
        let dir_on = TempDir::new().expect("tempdir");
        let ssb_on = Arc::new(Ssb::new(test_config(&dir_on), test_clock()).expect("ssb new"));

        let outcome = record_verdict(&ssb_on, "auditor-babysteps", 0, block_verdict(7))
            .await
            .expect("record_verdict must succeed");
        assert!(
            matches!(outcome, CascadeOutcome::RespawnOnce { .. }),
            "first BLOCK should respawn once, got {outcome:?}"
        );

        let rows_on = ssb_on
            .query_by_type(SsbActionType::AuditMastTagged)
            .expect("query_by_type must succeed");
        assert_eq!(
            rows_on.len(),
            1,
            "exactly one audit_mast_tagged row must be emitted for the BLOCK verdict when gate ON"
        );

        // ----- Leg 2: gate OFF -> verdict recorded, NO mast row -----
        std::env::remove_var("ELAI_FAILURE_TRACKER_ENABLE");
        let dir_off = TempDir::new().expect("tempdir");
        let ssb_off = Arc::new(Ssb::new(test_config(&dir_off), test_clock()).expect("ssb new"));

        record_verdict(&ssb_off, "auditor-babysteps", 0, block_verdict(8))
            .await
            .expect("record_verdict must succeed");

        let rows_off = ssb_off
            .query_by_type(SsbActionType::AuditMastTagged)
            .expect("query_by_type must succeed");
        assert!(
            rows_off.is_empty(),
            "no audit_mast_tagged row may be emitted when the gate is OFF (default)"
        );
    }

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

    // -------------------------------------------------------------------------
    // E-46 off-agent smoke: emit_goal_drift_for_verdict writes a readable
    // PlanningDecompositionEvent/GoalDriftCanaryBelowThreshold SSB row.
    //
    // R4-compliant: the test drives the REAL production fn
    // `emit_goal_drift_for_verdict` against a temp Ssb (not a shim calling the
    // inner emit fn directly), then reads back the row via
    // `ssb.query_by_type(SsbActionType::PlanningDecompositionEvent)`. The test
    // DID NOT author that row; the production code did.
    //
    // ENV NOTE: sets ELAI_PLANNING_DECOMPOSITION_ENABLE=1 then removes it.
    // Must not run in parallel with the E-6 test leg that removes
    // ELAI_FAILURE_TRACKER_ENABLE. Run sequentially with
    // `#[serial]` or isolate into separate test binaries. Here we use a
    // TempDir per invocation and sequential removal so there is no logical
    // ordering constraint — both tests write to their own isolated Ssb
    // databases.
    // -------------------------------------------------------------------------
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn emit_goal_drift_for_verdict_writes_readable_row_when_gate_on() {
        use crate::ssb::SsbActionType;

        std::env::set_var("ELAI_PLANNING_DECOMPOSITION_ENABLE", "1");
        let dir = TempDir::new().expect("tempdir");
        let ssb = Arc::new(Ssb::new(test_config(&dir), test_clock()).expect("ssb new"));

        // Use a high jaccard threshold (1.1, clamped to 1.0) so ANY non-empty
        // phase text is always below_threshold, guaranteeing the row is written.
        let below = emit_goal_drift_for_verdict(
            &ssb,
            "task-runner",
            0,
            "implement user authentication schema validation",
            "build a user authentication system with schema validation and JWT",
            false,
            1.1,   // jaccard_threshold -- clamped to 1.0 -> always below
            1.1,   // cosine_threshold  -- clamped to 1.0 -> always below
        )
        .await;

        assert!(below, "below_threshold must be true given threshold=1.0 and non-identical texts");

        let rows = ssb
            .query_by_type(SsbActionType::PlanningDecompositionEvent)
            .expect("query_by_type must succeed");
        assert!(
            !rows.is_empty(),
            "at least one PlanningDecompositionEvent row must be written when gate ON and below_threshold"
        );
        std::env::remove_var("ELAI_PLANNING_DECOMPOSITION_ENABLE");
    }
}
