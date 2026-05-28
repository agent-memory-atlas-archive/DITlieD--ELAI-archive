// E-10 contract: hostile-critic structural simplifications SSB variant.
//
// Verifies:
// 1. SCHEMA_VERSION_PUBLIC >= 163 (E-10 atomic bump for the single
//    CriticDecisionEvent variant with 9-arm CriticTilePhase discriminator).
// 2. CriticDecisionEvent round-trips via as_sql_str /
//    ssb_action_type_from_str_public.
// 3. action_type() on SsbPayload::CriticDecisionEvent returns the matching
//    SsbActionType.
// 4. CriticDecisionEventPayload serializes via msgpack (rmp_serde) without loss.
// 5. CriticTilePhase covers all 9 documented fire-points and the snake_case
//    wire labels match the dashboard tile mapping.

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, CriticDecisionEventPayload, CriticTilePhase, SsbActionType,
    SsbPayload, SCHEMA_VERSION_PUBLIC,
};

const E10_SCHEMA_FLOOR: i64 = 163;

#[test]
fn schema_version_includes_e10_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= E10_SCHEMA_FLOOR,
        "schema must include E-10 SSB variant (got {SCHEMA_VERSION_PUBLIC}, floor {E10_SCHEMA_FLOOR})"
    );
}

#[test]
fn critic_decision_event_action_type_roundtrip() {
    let at = SsbActionType::CriticDecisionEvent;
    let s = at.as_sql_str();
    assert_eq!(s, "critic_decision_event", "as_sql_str mismatch");
    let decoded = ssb_action_type_from_str_public(s).expect("from_str failed");
    assert_eq!(decoded, at, "round-trip failed for {s}");
}

#[test]
fn critic_tile_phase_wire_labels() {
    // 9 phases per DECISION-1; wire labels match dashboard tile mapping.
    let cases = [
        (CriticTilePhase::FsmAblation, "fsm_ablation"),
        (CriticTilePhase::RoleAblation, "role_ablation"),
        (CriticTilePhase::RagChallenger, "rag_challenger"),
        (CriticTilePhase::MistakeBookSignal, "mistake_book_signal"),
        (CriticTilePhase::TypedCotVerified, "typed_cot_verified"),
        (CriticTilePhase::TierMatrix, "tier_matrix"),
        (CriticTilePhase::CloudDefault, "cloud_default"),
        (CriticTilePhase::GrcBlockquote, "grc_blockquote"),
        (CriticTilePhase::PromptCacheHitRate, "prompt_cache_hit_rate"),
    ];
    for (phase, label) in cases {
        assert_eq!(phase.as_str(), label, "phase wire label mismatch");
    }
}

#[test]
fn critic_decision_event_payload_bijection() {
    let original = CriticDecisionEventPayload {
        phase: CriticTilePhase::FsmAblation,
        run_id_prefix: [0xE1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10],
        baseline_score: 0.62,
        challenger_score: 0.58,
        paired_bootstrap_p: Some(0.18),
        sample_size: 30,
        verdict_code: 0,
        rationale_hash: [0xE1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11],
        ts_ms: 1_700_000_163_001,
    };
    let payload = SsbPayload::CriticDecisionEvent(original.clone());
    assert_eq!(payload.action_type(), SsbActionType::CriticDecisionEvent);
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    if let SsbPayload::CriticDecisionEvent(p) = decoded {
        assert_eq!(p, original, "msgpack round-trip mismatch");
    } else {
        panic!("decoded wrong variant");
    }
}

#[test]
fn critic_decision_event_payload_optional_p() {
    // paired_bootstrap_p=None must survive the round-trip (DECISION-12
    // inline helper returns None for empty/mismatched inputs).
    let original = CriticDecisionEventPayload {
        phase: CriticTilePhase::RagChallenger,
        run_id_prefix: [0; 8],
        baseline_score: 0.0,
        challenger_score: 0.0,
        paired_bootstrap_p: None,
        sample_size: 0,
        verdict_code: 1,
        rationale_hash: [0; 8],
        ts_ms: 0,
    };
    let payload = SsbPayload::CriticDecisionEvent(original.clone());
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    if let SsbPayload::CriticDecisionEvent(p) = decoded {
        assert_eq!(p.paired_bootstrap_p, None);
        assert_eq!(p, original);
    } else {
        panic!("decoded wrong variant");
    }
}
