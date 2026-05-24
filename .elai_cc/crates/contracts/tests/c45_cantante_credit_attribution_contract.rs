//! C-45 — CANTANTE contrastive credit-attribution contract.
//!
//! Paper grounding: arXiv:2605.13295 (Zehle 2026, "CANTANTE: Optimizing
//! Agentic Systems via Contrastive Credit Attribution", Section 3.2 — K
//! rollouts per query NOT K^N; Section 4.4 footnote 1 — `K % g == 0`
//! partition divisibility; Appendix B.2 — g=3 sweet spot, g=2
//! catastrophic -54.4pp on GSM8K).
//!
//! Contract fences:
//!   G1. `DropPolicy` parses + serialises round-trip (rmp).
//!   G2. `contrastive_attribute` enforces K rollouts per configuration
//!       (NOT K^N) — total = K + K*N for N dropped roles.
//!   G3. Partition with `g=3` ships; partition with `g<=2` returns an
//!       error (paper-anchored guard).
//!   G4. The three new `SsbActionType::Credit*` variants round-trip
//!       through the SSB enum surface.
//!   G5. `SkillManifest.learned_credit_score` field exists + clamps to
//!       `[-1.0, +1.0]` via `update_credit_score`.
//!   G6. K-rollouts-NOT-K^N invariant via a counting harness (CANTANTE
//!       Section 3.2 efficiency claim).
//!
//! Build target: `cargo check -p elai_cc_contracts`. Test target:
//! `cargo test -p elai_cc_contracts --test c45_cantante_credit_attribution_contract`.

use std::sync::Arc;
use std::sync::Mutex;

use elai_orchestrator::credit_attribution::{
    contrastive_attribute, pilot_roles, validate_partition, CreditAttributionHarness,
    CreditReport, CreditScore, DropPolicy, HarnessError, PartitionError, G_DEFAULT,
};
use elai_orchestrator::ipc_types::AgentRole;
use elai_orchestrator::skill_registry::{SkillManifest, SkillOrigin};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, AgentRoleDroppedPayload, CreditAttributionResultPayload,
    CreditAttributionRunPayload, SsbActionType, SsbPayload,
};

// ---------------------------------------------------------------------------
// G1 — DropPolicy serde round-trip
// ---------------------------------------------------------------------------
#[test]
fn g1_drop_policy_round_trips_through_rmp() {
    let policy = DropPolicy::drop(AgentRole::Researcher);
    let bytes = rmp_serde::to_vec(&policy).expect("encode");
    let decoded: DropPolicy = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(decoded, policy);

    let baseline = DropPolicy::baseline();
    let bytes2 = rmp_serde::to_vec(&baseline).expect("encode baseline");
    let decoded2: DropPolicy = rmp_serde::from_slice(&bytes2).expect("decode baseline");
    assert_eq!(decoded2, baseline);
    assert!(decoded2.is_baseline());
}

// ---------------------------------------------------------------------------
// G2/G6 — K rollouts per configuration, NOT K^N (paper Section 3.2)
// ---------------------------------------------------------------------------
#[tokio::test]
async fn g2_contrastive_attribute_respects_k_rollouts_not_k_to_n() {
    let dropped = vec![
        AgentRole::Executor,
        AgentRole::Auditor,
        AgentRole::Researcher,
        AgentRole::Designer,
    ];
    let k: u32 = 9;
    let counter = Arc::new(Mutex::new(0_u32));
    let counter_clone = counter.clone();
    let _report = contrastive_attribute(&dropped, k, 0, move |_policy, _i| {
        let counter = counter_clone.clone();
        async move {
            *counter.lock().expect("lock") += 1;
            Ok::<f32, String>(0.5)
        }
    })
    .await
    .expect("attribute ok");

    let total_rollouts = *counter.lock().expect("lock");
    let expected = k + k * (dropped.len() as u32);
    assert_eq!(
        total_rollouts, expected,
        "CANTANTE Section 3.2 invariant: K rollouts per config (baseline + N drops), NOT K^N"
    );
    // K^N for K=9, N=4 would be 6561 — must be far below.
    assert!(
        total_rollouts < 9_u32.pow(4),
        "must not fan out K^N (cross-product blowup banned per paper Section 3.2)"
    );
}

// ---------------------------------------------------------------------------
// G3 — partition with g=3 ships; g<=2 errors (paper-anchored guard)
// ---------------------------------------------------------------------------
#[test]
fn g3_partition_g_three_ships_and_g_two_errors() {
    // Paper-anchored sweet spot.
    validate_partition(9, 3).expect("paper-default partition must validate");
    assert_eq!(G_DEFAULT, 3, "G_DEFAULT must equal the paper sweet spot");

    // Catastrophic per CANTANTE Appendix B.2 (-54.4pp on GSM8K).
    let err = validate_partition(6, 2).expect_err("g=2 must be refused");
    assert_eq!(err, PartitionError::GroupTooSmall { g: 2 });

    let err2 = validate_partition(9, 0).expect_err("g=0 must be refused");
    assert_eq!(err2, PartitionError::GroupTooSmall { g: 0 });

    // Indivisible partition refused.
    let err3 = validate_partition(10, 3).expect_err("K % g != 0 must be refused");
    assert_eq!(err3, PartitionError::Indivisible { k: 10, g: 3 });
}

#[tokio::test]
async fn g3_harness_refuses_g_two_at_run_time() {
    let harness =
        CreditAttributionHarness::new(4, 2, vec![AgentRole::Executor], 0);
    let res = harness
        .run(|_policy, _i| async { Ok::<f32, String>(0.5) })
        .await;
    assert!(matches!(
        res,
        Err(HarnessError::Partition(PartitionError::GroupTooSmall { g: 2 }))
    ));
}

// ---------------------------------------------------------------------------
// G4 — 3 SsbActionType::Credit* variants round-trip
// ---------------------------------------------------------------------------
#[test]
fn g4_agent_role_dropped_round_trips() {
    let t = SsbActionType::AgentRoleDropped;
    assert_eq!(t.as_sql_str(), "agent_role_dropped");
    let parsed = ssb_action_type_from_str_public("agent_role_dropped").expect("parse");
    assert_eq!(parsed, t);

    let payload = SsbPayload::AgentRoleDropped(AgentRoleDroppedPayload {
        role_name_hash: 0xDEAD_BEEF_CAFE_BABE,
        ts_ms: 1_748_044_900_000,
    });
    assert_eq!(payload.action_type(), t);
    let bytes = rmp_serde::to_vec(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(decoded, payload);
}

#[test]
fn g4_credit_attribution_run_round_trips() {
    let t = SsbActionType::CreditAttributionRun;
    assert_eq!(t.as_sql_str(), "credit_attribution_run");
    let parsed = ssb_action_type_from_str_public("credit_attribution_run").expect("parse");
    assert_eq!(parsed, t);

    let payload = SsbPayload::CreditAttributionRun(CreditAttributionRunPayload {
        run_id_hash: 0x1111_2222_3333_4444,
        dropped_role_name_hash: 0xAAAA_BBBB_CCCC_DDDD,
        k_rollouts: 9,
        g_partition: 3,
        ts_ms: 1_748_044_900_000,
    });
    assert_eq!(payload.action_type(), t);
    let bytes = rmp_serde::to_vec(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(decoded, payload);
}

#[test]
fn g4_credit_attribution_result_round_trips() {
    let t = SsbActionType::CreditAttributionResult;
    assert_eq!(t.as_sql_str(), "credit_attribution_result");
    let parsed = ssb_action_type_from_str_public("credit_attribution_result").expect("parse");
    assert_eq!(parsed, t);

    let payload = SsbPayload::CreditAttributionResult(CreditAttributionResultPayload {
        run_id_hash: 0x1111_2222_3333_4444,
        per_role_scores_hash: 0x5555_6666_7777_8888,
        total_roles: 8,
        ts_ms: 1_748_044_900_000,
    });
    assert_eq!(payload.action_type(), t);
    let bytes = rmp_serde::to_vec(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(decoded, payload);
}

// ---------------------------------------------------------------------------
// G5 — SkillManifest.learned_credit_score field exists + clamps
// ---------------------------------------------------------------------------
#[test]
fn g5_skill_manifest_carries_learned_credit_score_field() {
    let m = SkillManifest {
        name: "x".into(),
        version: "0.0.0".into(),
        triggers: vec![],
        inputs: vec![],
        outputs: vec![],
        scope: String::new(),
        origin: SkillOrigin::SkillsDir,
        file_path: std::path::PathBuf::from("x"),
        learned_credit_score: None,
        distilled_fragments: Vec::new(),
    };
    // Field defaults to None for back-compat with pre-C-45 manifests.
    assert_eq!(m.learned_credit_score, None);
}

#[test]
fn g5_update_credit_score_clamps_paper_anchored_range() {
    let mut m = SkillManifest {
        name: "x".into(),
        version: "0.0.0".into(),
        triggers: vec![],
        inputs: vec![],
        outputs: vec![],
        scope: String::new(),
        origin: SkillOrigin::SkillsDir,
        file_path: std::path::PathBuf::from("x"),
        learned_credit_score: None,
        distilled_fragments: Vec::new(),
    };
    // Above +1.0 clamps to +1.0.
    m.update_credit_score(7.5);
    assert_eq!(m.learned_credit_score, Some(1.0));
    // Below -1.0 clamps to -1.0.
    m.update_credit_score(-3.2);
    assert_eq!(m.learned_credit_score, Some(-1.0));
    // In-band passes through.
    m.update_credit_score(0.42);
    assert_eq!(m.learned_credit_score, Some(0.42));
    // NaN clamps to zero.
    m.update_credit_score(f32::NAN);
    assert_eq!(m.learned_credit_score, Some(0.0));
}

// ---------------------------------------------------------------------------
// Pilot-set + report shape sanity
// ---------------------------------------------------------------------------
#[test]
fn pilot_roles_is_eight_base_roles_per_plan_rebase() {
    let roles = pilot_roles();
    assert_eq!(
        roles.len(),
        8,
        "C-45 pilot ablates 8 base roles only (cost-cap; B-40/B-41 roles excluded)"
    );
    // B-40/B-41/E-12 expansions intentionally excluded.
    assert!(!roles.contains(&AgentRole::IdeaFramer));
    assert!(!roles.contains(&AgentRole::IdeaCritic));
    assert!(!roles.contains(&AgentRole::AuditorConcepts));
    assert!(!roles.contains(&AgentRole::AuditorPhases));
    assert!(!roles.contains(&AgentRole::AuditorSteps));
    assert!(!roles.contains(&AgentRole::AuditorBabySteps));
    assert!(!roles.contains(&AgentRole::AuditorBabyStepDetail));
    assert!(!roles.contains(&AgentRole::AuditorProductionGrade));
    // E-37: injection classifier excluded from C-45 pilot ablation (cost-cap;
    // off-model classifier not in scope for credit attribution pilot).
    assert!(!roles.contains(&AgentRole::InjectionClassifier));
}

#[tokio::test]
async fn pilot_harness_emits_credit_report_with_eight_roles() {
    let harness = CreditAttributionHarness::pilot(1_700_000_000_000);
    let report: CreditReport = harness
        .run(|policy, _i| async move {
            // Synthetic: baseline 0.8, drops 0.5 (positive credit for all).
            if policy.is_baseline() {
                Ok::<f32, String>(0.8)
            } else {
                Ok::<f32, String>(0.5)
            }
        })
        .await
        .expect("pilot harness ok");
    assert_eq!(report.len(), 8);
    // Per-role credit clamped to [-1, +1].
    for role in pilot_roles() {
        let c = report.get(*role).expect("role present");
        assert!(c.is_finite());
        assert!(c >= -1.0 && c <= 1.0);
    }
}

#[test]
fn credit_score_paper_anchored_clamp_holds_at_boundaries() {
    // CANTANTE Section 3.2: per-rollout credit c in [-1.0, +1.0].
    assert!((CreditScore::clamped(0.0).value() - 0.0).abs() < f32::EPSILON);
    assert!((CreditScore::clamped(1.0).value() - 1.0).abs() < f32::EPSILON);
    assert!((CreditScore::clamped(-1.0).value() - -1.0).abs() < f32::EPSILON);
    assert!((CreditScore::clamped(1.5).value() - 1.0).abs() < f32::EPSILON);
    assert!((CreditScore::clamped(-1.5).value() - -1.0).abs() < f32::EPSILON);
    assert!((CreditScore::clamped(f32::NAN).value() - 0.0).abs() < f32::EPSILON);
    assert!((CreditScore::clamped(f32::INFINITY).value() - 0.0).abs() < f32::EPSILON);
}
