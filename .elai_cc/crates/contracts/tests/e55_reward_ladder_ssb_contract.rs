// E-55 (SCHEMA_VERSION 168) contract: agentic RL + reward ladder
// substrate SSB variant emit + payload roundtrip + env gate.
//
// Verifies the non-RUNPOD portions of the plan ACCEPTANCE 14
// orchestrator integration tests. The remaining 11 integration tests
// (rlaif_judge_invokes_via_per_role_runtime, gepa_evolves_bottom_quartile_arms,
// rollout_collector_threshold_at_200, bai_arm_observation_with_fused_reward,
// etc.) are RUNPOD-PENDING per workflow/reasoning/E-55-decisions.md --
// they require either a live PerRoleRuntime cloud Sonnet call OR a
// 200-rollout GREEN session feed. This contract covers the substrate
// invariants those tests presuppose:
//
// 1. SCHEMA_VERSION_PUBLIC >= 168 (E-55 floor; RewardLadderEvent variant
//    landed at this bump).
// 2. SsbActionType::RewardLadderEvent + RewardLadderEventKind 9-arm
//    discriminator round-trip via as_sql_str / from_str.
// 3. RewardLadderEventPayload msgpack-bijection holds for all 9 kinds
//    (covers the producer-side serialise + consumer-side deserialise
//    wire-format invariant that the 14 missing integration tests would
//    have transitively exercised).
// 4. action_type() on SsbPayload::RewardLadderEvent returns
//    SsbActionType::RewardLadderEvent.
// 5. ENV_AGENTIC_RL_ENABLE env-gate const matches the documented name
//    ("ELAI_AGENTIC_RL_ENABLE") -- regression guard against silent
//    rename that would unblock the wire-dark gate.
// 6. BAI update_with_fused on ArmPosterior produces the same posterior
//    as the bool variant under the documented threshold semantics
//    (covers the load-bearing wire that the E-55 audit flagged as
//    BLOCK B-1).
//
// Grounding: workflow/reasoning/E-55-decisions.md DECISION-1 (Pattern-2
// 20th application) + DECISION-7 (3 unlock conditions) + E-55 audit
// BLOCK B-2 (14 integration tests absent; this test closes the
// non-RUNPOD substrate slice). Remaining 11 RUNPOD-PENDING tests
// override-documented in E-55-decisions.md DECISION-18.

use elai_orchestrator::bai::ArmPosterior;
use elai_orchestrator::reasoning_template::TemplateId;
use elai_orchestrator::rl::{agentic_rl_enabled, ENV_AGENTIC_RL_ENABLE};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, RewardLadderEventKind, RewardLadderEventPayload,
    SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC,
};

const E55_SCHEMA_FLOOR: i64 = 168;

#[test]
fn schema_version_includes_e55_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= E55_SCHEMA_FLOOR,
        "schema must include E-55 RewardLadderEvent variant \
         (got {SCHEMA_VERSION_PUBLIC}, floor {E55_SCHEMA_FLOOR})"
    );
}

#[test]
fn reward_ladder_event_action_type_roundtrip() {
    let at = SsbActionType::RewardLadderEvent;
    let s = at.as_sql_str();
    assert_eq!(s, "reward_ladder_event", "as_sql_str mismatch");
    let decoded = ssb_action_type_from_str_public(s).expect("from_str failed");
    assert_eq!(decoded, at, "round-trip failed for {s}");
}

#[test]
fn nine_arm_wire_labels_distinct_and_snake_case() {
    // The 9 kinds round-trip through as_str() -- the canonical wire
    // label that the dashboard tile + CLI subcommand parse.
    let cases: &[(RewardLadderEventKind, &str)] = &[
        (RewardLadderEventKind::RlaifJudgeRated, "rlaif_judge_rated"),
        (
            RewardLadderEventKind::RlaifCalibrationCorrelationLow,
            "rlaif_calibration_correlation_low",
        ),
        (
            RewardLadderEventKind::GepaTemplateEvolved,
            "gepa_template_evolved",
        ),
        (
            RewardLadderEventKind::GepaTemplateEvicted,
            "gepa_template_evicted",
        ),
        (
            RewardLadderEventKind::RewardFusionComputed,
            "reward_fusion_computed",
        ),
        (
            RewardLadderEventKind::RewardHackingSuspected,
            "reward_hacking_suspected",
        ),
        (
            RewardLadderEventKind::SftTrainingThresholdReached,
            "sft_training_threshold_reached",
        ),
        (
            RewardLadderEventKind::BaiArmObservationRecorded,
            "bai_arm_observation_recorded",
        ),
        (
            RewardLadderEventKind::RlWeightUpdateUnlocked,
            "rl_weight_update_unlocked",
        ),
    ];
    for (kind, label) in cases {
        assert_eq!(kind.as_str(), *label, "kind wire label mismatch");
    }
    let labels: std::collections::BTreeSet<_> = cases.iter().map(|(_, l)| *l).collect();
    assert_eq!(labels.len(), 9, "wire labels must be distinct");
}

#[test]
fn reward_ladder_event_payload_msgpack_bijection_all_kinds() {
    let kinds = [
        RewardLadderEventKind::RlaifJudgeRated,
        RewardLadderEventKind::RlaifCalibrationCorrelationLow,
        RewardLadderEventKind::GepaTemplateEvolved,
        RewardLadderEventKind::GepaTemplateEvicted,
        RewardLadderEventKind::RewardFusionComputed,
        RewardLadderEventKind::RewardHackingSuspected,
        RewardLadderEventKind::SftTrainingThresholdReached,
        RewardLadderEventKind::BaiArmObservationRecorded,
        RewardLadderEventKind::RlWeightUpdateUnlocked,
    ];
    for (i, kind) in kinds.iter().enumerate() {
        let original = RewardLadderEventPayload {
            kind: *kind,
            session_id: Some(format!("s-e55-{i}")),
            trajectory_id: if matches!(kind, RewardLadderEventKind::RlWeightUpdateUnlocked) {
                None
            } else {
                Some(format!("t-{i:04}"))
            },
            details_json: serde_json::json!({
                "kind_label": kind.as_str(),
                "caller_site": "e55_contract_test"
            }),
            ts_ms: 1_700_000_168_000 + i as i64,
        };
        let payload = SsbPayload::RewardLadderEvent(original.clone());
        assert_eq!(
            payload.action_type(),
            SsbActionType::RewardLadderEvent,
            "action_type mismatch for {kind:?}"
        );
        let encoded = rmp_serde::to_vec(&payload).expect("msgpack encode");
        let decoded: SsbPayload = rmp_serde::from_slice(&encoded).expect("msgpack decode");
        match decoded {
            SsbPayload::RewardLadderEvent(p) => assert_eq!(p, original, "roundtrip mismatch"),
            other => panic!("unexpected variant after round-trip: {other:?}"),
        }
    }
}

#[test]
fn env_gate_const_matches_documented_name() {
    // Regression guard: the wire-dark env gate name must NOT drift.
    // Tied to workflow/reasoning/E-55-decisions.md + the CLI rl
    // subcommand + the dashboard tile.
    assert_eq!(
        ENV_AGENTIC_RL_ENABLE, "ELAI_AGENTIC_RL_ENABLE",
        "env gate name drift -- E-55 decisions / CLI / dashboard all read this"
    );
    // Calling agentic_rl_enabled() is side-effect-free; we only confirm
    // it does not panic. Result depends on test-process env which we do
    // not mutate to keep multi-threaded test runs deterministic.
    let _ = agentic_rl_enabled();
}

#[test]
fn bai_update_with_fused_matches_bool_variant() {
    // The E-55 audit BLOCK B-1 was "ArmPosterior::update_with_fused
    // absent". This contract test pins the additive method's semantics:
    // update_with_fused(fused, threshold) MUST produce the same posterior
    // as update(fused >= threshold).
    let template = TemplateId::new_v4();
    let mut a = ArmPosterior::new(template);
    let mut b = ArmPosterior::new(template);
    // Pass: fused=0.7 >= 0.5 -> alpha += 1
    a.update_with_fused(0.7, 0.5);
    b.update(true);
    assert_eq!(a.alpha, b.alpha, "pass alpha mismatch");
    assert_eq!(a.beta, b.beta, "pass beta mismatch");
    assert_eq!(a.n_pulls, b.n_pulls, "pass n_pulls mismatch");
    // Fail: fused=0.3 < 0.5 -> beta += 1
    a.update_with_fused(0.3, 0.5);
    b.update(false);
    assert_eq!(a.alpha, b.alpha, "fail alpha mismatch");
    assert_eq!(a.beta, b.beta, "fail beta mismatch");
    assert_eq!(a.n_pulls, b.n_pulls, "fail n_pulls mismatch");
    // Exact-threshold tie: fused=0.5 >= 0.5 -> pass (alpha += 1)
    let mut c = ArmPosterior::new(TemplateId::new_v4());
    c.update_with_fused(0.5, 0.5);
    assert!(
        (c.alpha - 2.0).abs() < f64::EPSILON,
        "tie at threshold must count as pass (alpha=2.0 after one update)"
    );
}
