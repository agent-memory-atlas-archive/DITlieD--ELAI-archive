//! B-41 Slice 3i — Per-Level Audit contract tests.
//!
//! Exercises the `per_level_audit_dispatch::record_verdict` cascade policy
//! end-to-end against a real SSB writer. Covers:
//!
//!   1. Allow verdict -> Pass, exactly one PerLevelAuditVerdict row written.
//!   2. First BLOCK -> RespawnOnce, respawned_once=true on the returned
//!      verdict, exactly two PerLevelAuditVerdict rows (original + respawn-
//!      marked) and ZERO PerLevelAuditCascade rows.
//!   3. Second BLOCK (respawned_once already true) -> Cascade, cascade_to_
//!      _parent=true on the returned verdict, exactly one PerLevelAuditCascade
//!      row, parent_tier matches the dispatcher mapping.
//!   4. BabyStepDetail second-BLOCK cascades to "baby_step" (DECISION-CASCADE).
//!   5. SSB persistence: all 3 new action_type strings round-trip through
//!      ssb_action_type_from_str + as_sql_str + SsbPayload::action_type.

use elai_core::clock::SeededClock;
use elai_orchestrator::auditor_mode::{
    mark_respawned, AuditorMode, Finding, LevelAuditVerdict, Severity, TierNodeId, Verdict,
};
use elai_orchestrator::per_level_audit_dispatch::{
    emit_started, record_verdict, CascadeOutcome,
};
use elai_orchestrator::research_tree::{BabyStepId, ConceptId, PhaseId, StepId};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader,
    SCHEMA_VERSION_PUBLIC,
};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

fn make_ssb() -> (Arc<Ssb>, TempDir) {
    let dir = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: dir.path().join("plac.db"),
        pool_size: Some(2),
        project_id: "p-test".into(),
        session_id: "s-test".into(),
    };
    let clock = Arc::new(SeededClock::with_epoch(
        SystemTime::UNIX_EPOCH + Duration::from_millis(1_700_000_000_000),
    )) as Arc<dyn elai_core::ClockProvider>;
    let ssb = Ssb::new(cfg, clock).expect("ssb init");
    (Arc::new(ssb), dir)
}

#[test]
fn contract_schema_version_bumped_to_45() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 45,
        "B-41 requires SCHEMA_VERSION >= 45 (got {SCHEMA_VERSION_PUBLIC})"
    );
}

#[test]
fn contract_three_action_type_strings_round_trip() {
    for (s, want) in [
        ("per_level_audit_started", SsbActionType::PerLevelAuditStarted),
        ("per_level_audit_verdict", SsbActionType::PerLevelAuditVerdict),
        ("per_level_audit_cascade", SsbActionType::PerLevelAuditCascade),
    ] {
        let got = ssb_action_type_from_str_public(s).expect("parse");
        assert_eq!(got, want, "string -> enum mismatch for {s}");
        assert_eq!(got.as_sql_str(), s, "enum -> string mismatch for {s}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_allow_verdict_returns_pass_and_emits_one_row() {
    let (ssb, _dir) = make_ssb();
    let v = LevelAuditVerdict::allow(AuditorMode::Concepts, TierNodeId::Concept(ConceptId(1)));
    let outcome = record_verdict(&ssb, "auditor-test", 0, v)
        .await
        .expect("record_verdict ok");
    assert_eq!(outcome, CascadeOutcome::Pass);
    let rows = ssb
        .query_by_type(SsbActionType::PerLevelAuditVerdict)
        .expect("query");
    assert_eq!(rows.len(), 1, "Allow path writes exactly one verdict row");
    let cascade_rows = ssb
        .query_by_type(SsbActionType::PerLevelAuditCascade)
        .expect("query");
    assert!(cascade_rows.is_empty(), "Allow must not cascade");
}

fn block_verdict(mode: AuditorMode, tier: TierNodeId) -> LevelAuditVerdict {
    LevelAuditVerdict::from_findings(
        mode,
        tier,
        vec![Finding {
            category: "C-TEST".into(),
            severity: Severity::Block,
            message: "test-block".into(),
            source: None,
        }],
    )
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_first_block_respawns_once_no_cascade() {
    let (ssb, _dir) = make_ssb();
    let v = block_verdict(AuditorMode::Steps, TierNodeId::Step(StepId(7)));
    let outcome = record_verdict(&ssb, "auditor-test", 0, v)
        .await
        .expect("record_verdict ok");
    match outcome {
        CascadeOutcome::RespawnOnce { verdict } => {
            assert!(verdict.respawned_once, "respawned_once must flip to true");
            assert!(!verdict.cascade_to_parent, "must not cascade on first block");
            assert_eq!(verdict.verdict, Verdict::Block);
        }
        other => panic!("expected RespawnOnce, got {other:?}"),
    }
    let verdict_rows = ssb
        .query_by_type(SsbActionType::PerLevelAuditVerdict)
        .expect("query");
    assert_eq!(
        verdict_rows.len(),
        2,
        "first block emits two verdict rows (original + respawn-marked)"
    );
    let cascade_rows = ssb
        .query_by_type(SsbActionType::PerLevelAuditCascade)
        .expect("query");
    assert!(cascade_rows.is_empty(), "first block must not cascade");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_second_block_cascades_to_parent() {
    let (ssb, _dir) = make_ssb();
    let mut v = block_verdict(AuditorMode::Phases, TierNodeId::Phase(PhaseId(3)));
    v.respawned_once = true; // simulate "already respawned once".
    let outcome = record_verdict(&ssb, "auditor-test", 0, v)
        .await
        .expect("record_verdict ok");
    match outcome {
        CascadeOutcome::Cascade {
            verdict,
            parent_tier,
        } => {
            assert!(verdict.cascade_to_parent);
            assert_eq!(parent_tier, "concept", "Phase parent is concept");
        }
        other => panic!("expected Cascade, got {other:?}"),
    }
    let cascade_rows = ssb
        .query_by_type(SsbActionType::PerLevelAuditCascade)
        .expect("query");
    assert_eq!(cascade_rows.len(), 1, "second block emits one cascade row");
    // Confirm payload carries the cascade_to_parent=true verdict.
    let last_verdict = ssb
        .query_by_type(SsbActionType::PerLevelAuditVerdict)
        .expect("query");
    assert!(!last_verdict.is_empty());
    let payload: SsbPayload =
        rmp_serde::from_slice(&last_verdict.last().expect("last").payload).expect("decode");
    match payload {
        SsbPayload::PerLevelAuditVerdict(p) => {
            assert!(p.cascade_to_parent, "final verdict row carries cascade=true");
            assert_eq!(p.verdict, "block");
        }
        other => panic!("expected PerLevelAuditVerdict, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_babystep_detail_cascades_to_baby_step() {
    let (ssb, _dir) = make_ssb();
    let mut v = block_verdict(
        AuditorMode::BabyStepDetail,
        TierNodeId::BabyStep(BabyStepId(42)),
    );
    v.respawned_once = true;
    let outcome = record_verdict(&ssb, "auditor-test", 0, v)
        .await
        .expect("record_verdict ok");
    match outcome {
        CascadeOutcome::Cascade { parent_tier, .. } => {
            assert_eq!(
                parent_tier, "baby_step",
                "B-41-decisions DECISION-CASCADE: BabyStepDetail cascades to BabyStep"
            );
        }
        other => panic!("expected Cascade, got {other:?}"),
    }
    let cascade_rows = ssb
        .query_by_type(SsbActionType::PerLevelAuditCascade)
        .expect("query");
    let payload: SsbPayload =
        rmp_serde::from_slice(&cascade_rows.last().expect("row").payload).expect("decode");
    match payload {
        SsbPayload::PerLevelAuditCascade(p) => {
            assert_eq!(p.mode, "babystep_detail");
            assert_eq!(p.tier, "baby_step");
            assert_eq!(p.parent_tier, "baby_step");
            assert_eq!(p.node_id, 42);
        }
        other => panic!("expected cascade payload, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_started_row_writes_correct_labels() {
    let (ssb, _dir) = make_ssb();
    // Use the dispatcher's emit_started via record_verdict's prefix: we
    // simply re-emit by calling emit_started directly through the public API.
    emit_started(
        &ssb,
        "auditor-test",
        0,
        AuditorMode::BabySteps,
        TierNodeId::BabyStep(BabyStepId(9)),
    )
    .await
    .expect("emit_started ok");
    let rows = ssb
        .query_by_type(SsbActionType::PerLevelAuditStarted)
        .expect("query");
    assert_eq!(rows.len(), 1);
    let payload: SsbPayload =
        rmp_serde::from_slice(&rows.last().expect("row").payload).expect("decode");
    match payload {
        SsbPayload::PerLevelAuditStarted(p) => {
            assert_eq!(p.mode, "babysteps");
            assert_eq!(p.tier, "baby_step");
            assert_eq!(p.node_id, 9);
        }
        other => panic!("expected started payload, got {other:?}"),
    }
}

/// B-41 audit BLOCK-2: Concepts-tier second BLOCK MUST escalate to FRAMING
/// (idea-level re-framing) instead of returning a self-loop Cascade row.
/// Asserts: outcome is `EscalateToIdea`, the cascade SSB row carries
/// `parent_tier="idea"` so the navigator / dashboard can route to the
/// FRAMING substate retry (B-40) rather than infinitely respawning the
/// Concept Designer.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_second_block_at_concepts_escalates_to_idea() {
    let (ssb, _dir) = make_ssb();
    let mut v = block_verdict(AuditorMode::Concepts, TierNodeId::Concept(ConceptId(1)));
    v.respawned_once = true; // 2nd BLOCK at the top tier.
    let outcome = record_verdict(&ssb, "auditor-test", 0, v)
        .await
        .expect("record_verdict ok");
    match outcome {
        CascadeOutcome::EscalateToIdea { verdict } => {
            assert!(verdict.cascade_to_parent);
            assert_eq!(verdict.verdict, Verdict::Block);
            assert_eq!(verdict.mode, AuditorMode::Concepts);
        }
        CascadeOutcome::Cascade { parent_tier, .. } => panic!(
            "Concepts top-tier 2nd BLOCK must NOT return Cascade (parent={parent_tier:?}) — \
             that would self-loop. Expected EscalateToIdea."
        ),
        other => panic!("expected EscalateToIdea, got {other:?}"),
    }
    let cascade_rows = ssb
        .query_by_type(SsbActionType::PerLevelAuditCascade)
        .expect("query");
    assert_eq!(
        cascade_rows.len(),
        1,
        "top-tier escalation still emits exactly one cascade row (with parent_tier=idea)"
    );
    let payload: SsbPayload =
        rmp_serde::from_slice(&cascade_rows.last().expect("row").payload).expect("decode");
    match payload {
        SsbPayload::PerLevelAuditCascade(p) => {
            assert_eq!(p.mode, "concepts");
            assert_eq!(p.tier, "concept");
            assert_eq!(
                p.parent_tier, "idea",
                "top-tier escalation must use parent_tier=idea so dashboard / \
                 navigator can route to FRAMING retry instead of respawning Designer"
            );
        }
        other => panic!("expected cascade payload, got {other:?}"),
    }
}

/// B-41 audit WARN-2: full 5-step SSB sequence for a respawn-then-recover
/// trajectory at a single tier node. Asserts ordering, row counts, and
/// the respawned_once flag carrying through to the post-respawn verdict
/// row.
///
/// Sequence:
///   1. started(respawned_once=false implicit) — auditor first dispatch.
///   2. verdict(BLOCK, respawned_once=false) — first finding row.
///   3. mark_respawned (no SSB write — pure state mutation that record_verdict
///      already emitted as a second verdict row in step 1's record_verdict
///      call). To exercise the full 5-row sequence we manually re-emit the
///      started row for the respawn pass.
///   4. started(respawned_once=true implicit) — auditor respawn dispatch.
///   5. verdict(ALLOW) — recovery row.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_observable_effect_full_sequence_respawn_then_recover() {
    let (ssb, _dir) = make_ssb();
    let tier = TierNodeId::Step(StepId(11));
    let mode = AuditorMode::Steps;

    // Row 1 — initial started.
    emit_started(&ssb, "auditor-test", 0, mode, tier)
        .await
        .expect("emit_started 1");

    // Row 2 + Row 3 — record_verdict on a fresh BLOCK emits the original
    // verdict row AND a respawn-marked verdict row (per dispatcher
    // contract: see record_verdict step 3 in per_level_audit_dispatch.rs).
    let v_block = block_verdict(mode, tier);
    let outcome = record_verdict(&ssb, "auditor-test", 0, v_block)
        .await
        .expect("record_verdict block");
    let respawn_marked = match outcome {
        CascadeOutcome::RespawnOnce { verdict } => verdict,
        other => panic!("expected RespawnOnce, got {other:?}"),
    };
    assert!(respawn_marked.respawned_once);

    // Mark_respawned is also idempotent at the type level (used here
    // defensively to surface the contract — the verdict from
    // CascadeOutcome::RespawnOnce already has respawned_once=true).
    assert!(mark_respawned(LevelAuditVerdict::allow(mode, tier))
        .expect("mark_respawned on fresh")
        .respawned_once);

    // Row 4 — respawn started (caller convention emits a second started
    // row before the respawn auditor agent fires).
    emit_started(&ssb, "auditor-test", 1, mode, tier)
        .await
        .expect("emit_started 2");

    // Row 5 — recovery verdict (ALLOW with respawned_once=true preserved).
    let mut v_allow = LevelAuditVerdict::allow(mode, tier);
    v_allow.respawned_once = true;
    let recovery_outcome = record_verdict(&ssb, "auditor-test", 1, v_allow)
        .await
        .expect("record_verdict allow");
    assert_eq!(recovery_outcome, CascadeOutcome::Pass);

    // Final tally — 2 started + 3 verdict (BLOCK + respawn-marked + ALLOW)
    // + 0 cascade rows. The brief calls this a "5-step sequence"; the
    // dispatcher contract emits 2 verdict rows for the first BLOCK so the
    // observable SSB stream has 5 rows total in this trajectory.
    let started_rows = ssb
        .query_by_type(SsbActionType::PerLevelAuditStarted)
        .expect("query started");
    assert_eq!(started_rows.len(), 2, "two started rows (initial + respawn)");
    let verdict_rows = ssb
        .query_by_type(SsbActionType::PerLevelAuditVerdict)
        .expect("query verdict");
    assert_eq!(
        verdict_rows.len(),
        3,
        "three verdict rows (BLOCK + respawn-marked BLOCK + recovery ALLOW)"
    );
    let cascade_rows = ssb
        .query_by_type(SsbActionType::PerLevelAuditCascade)
        .expect("query cascade");
    assert!(
        cascade_rows.is_empty(),
        "no cascade rows on a respawn-then-recover trajectory"
    );

    // Ordering: started_rows[0].seq < verdict_rows[0..=1].seq < started_rows[1].seq
    // < verdict_rows[2].seq.
    assert!(started_rows[0].seq < verdict_rows[0].seq);
    assert!(verdict_rows[1].seq < started_rows[1].seq);
    assert!(started_rows[1].seq < verdict_rows[2].seq);

    // Decode the respawn-marked verdict row (row index 1 in verdict_rows)
    // and assert respawned_once=true carries through.
    let respawn_payload: SsbPayload =
        rmp_serde::from_slice(&verdict_rows[1].payload).expect("decode respawn-marked");
    match respawn_payload {
        SsbPayload::PerLevelAuditVerdict(p) => {
            assert!(
                p.respawned_once,
                "respawn-marked verdict row must carry respawned_once=true"
            );
            assert_eq!(p.verdict, "block");
        }
        other => panic!("expected verdict payload, got {other:?}"),
    }
    // Decode the recovery verdict row (index 2) and assert ALLOW with
    // respawned_once still true (the flag is monotonic).
    let recovery_payload: SsbPayload =
        rmp_serde::from_slice(&verdict_rows[2].payload).expect("decode recovery");
    match recovery_payload {
        SsbPayload::PerLevelAuditVerdict(p) => {
            assert_eq!(p.verdict, "allow");
            assert!(p.respawned_once);
            assert!(!p.cascade_to_parent);
        }
        other => panic!("expected verdict payload, got {other:?}"),
    }
}
