// E-19 contract: reasoning quality probe SSB variant.
//
// Verifies:
// 1. SCHEMA_VERSION_PUBLIC >= 166 (E-19 atomic bump for the single
//    ReasoningQualityProbe variant with 5-arm ProbeKind discriminator).
// 2. ReasoningQualityProbe round-trips via as_sql_str /
//    ssb_action_type_from_str_public.
// 3. action_type() on SsbPayload::ReasoningQualityProbe returns the matching
//    SsbActionType.
// 4. ReasoningQualityProbePayload serializes via msgpack (rmp_serde) without
//    loss for every ProbeKind arm.
// 5. ProbeKind covers all 5 documented fire-points and the snake_case wire
//    labels match the dashboard panel mapping.
//
// See workflow/reasoning/E-19-decisions.md.

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, ProbeKind, ReasoningQualityProbePayload, SsbActionType,
    SsbPayload, SCHEMA_VERSION_PUBLIC,
};

const E19_SCHEMA_FLOOR: i64 = 166;

#[test]
fn schema_version_includes_e19_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= E19_SCHEMA_FLOOR,
        "schema must include E-19 SSB variant (got {SCHEMA_VERSION_PUBLIC}, floor {E19_SCHEMA_FLOOR})"
    );
}

#[test]
fn reasoning_quality_probe_action_type_roundtrip() {
    let at = SsbActionType::ReasoningQualityProbe;
    let s = at.as_sql_str();
    assert_eq!(s, "reasoning_quality_probe", "as_sql_str mismatch");
    let decoded = ssb_action_type_from_str_public(s).expect("from_str failed");
    assert_eq!(decoded, at, "round-trip failed for {s}");
}

#[test]
fn probe_kind_wire_labels() {
    // 5 probe kinds per DECISION-PATTERN2; wire labels match the dashboard
    // panel mapping and the orchestrator dispatcher emission sites.
    let cases = [
        (ProbeKind::L2GateDecision, "l2_gate_decision"),
        (ProbeKind::L4PrmLiteScore, "l4_prm_lite_score"),
        (ProbeKind::L4LateralKept, "l4_lateral_kept"),
        (ProbeKind::L4TtcBudgetChosen, "l4_ttc_budget_chosen"),
        (ProbeKind::L6VerifierDiversity, "l6_verifier_diversity"),
    ];
    for (kind, label) in cases {
        assert_eq!(kind.as_str(), label, "kind wire label mismatch");
    }
}

#[test]
fn reasoning_quality_probe_payload_bijection() {
    // Round-trip every ProbeKind arm + payload combo via msgpack.
    let arms = [
        (
            ProbeKind::L2GateDecision,
            2_u8,
            "skip_high_confidence",
            Some(0.91_f32),
            Some(0.80_f32),
            None,
            None,
            "missing_import",
            "no_cot_class=false",
        ),
        (
            ProbeKind::L4PrmLiteScore,
            4,
            "weighted",
            Some(0.74),
            None,
            None,
            None,
            "",
            "auditor_pass=true|mean_logprob=0.6|cosine=0.7",
        ),
        (
            ProbeKind::L4LateralKept,
            4,
            "kept_one_extra_ply",
            Some(0.22),
            None,
            None,
            None,
            "",
            "branch_id=cand-7|utility=0.22|consistent=true",
        ),
        (
            ProbeKind::L4TtcBudgetChosen,
            4,
            "hard",
            None,
            None,
            None,
            None,
            "cross_lang_drift",
            "depth=3|n_candidates=5|n_lateral=2",
        ),
        (
            ProbeKind::L6VerifierDiversity,
            6,
            "disagree",
            None,
            None,
            None,
            None,
            "",
            "fresh_seed=true|rotated_l3=false|reason=rotated_l3_disagreed",
        ),
    ];

    for (kind, layer, verdict, score, threshold, observed_tokens, token_cap, ftc, extras) in arms {
        let original = ReasoningQualityProbePayload {
            kind,
            layer,
            baby_step_id: "bs-e19-contract".into(),
            verdict: verdict.into(),
            score,
            threshold,
            observed_tokens,
            token_cap,
            failure_taxonomy_category: ftc.into(),
            extras: extras.into(),
            ts_ms: 1_700_000_166_001,
        };
        let payload = SsbPayload::ReasoningQualityProbe(original.clone());
        assert_eq!(payload.action_type(), SsbActionType::ReasoningQualityProbe);
        let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
        let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
        if let SsbPayload::ReasoningQualityProbe(p) = decoded {
            assert_eq!(p, original, "msgpack round-trip mismatch for {}", kind.as_str());
        } else {
            panic!("decoded wrong variant for {}", kind.as_str());
        }
    }
}
