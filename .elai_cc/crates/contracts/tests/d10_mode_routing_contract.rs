// D-10 contract: mode-routing predicate + ClaimRouted SSB beacon.
//
// Verifies:
// 1. SCHEMA_VERSION_PUBLIC >= 77 (atomic single-bump for the D-10 ClaimRouted variant).
// 2. New SsbActionType::ClaimRouted round-trips via as_sql_str /
//    ssb_action_type_from_str_public.
// 3. action_type() on the matching SsbPayload variant returns the same
//    SsbActionType (preserves the bijection asserted by
//    gotcha B41_SSB_ALL_TEST_PAYLOADS_DRIFT).
// 4. ClaimRoutedPayload serializes via msgpack (rmp_serde) AND JSON
//    (serde_json) without loss.
// 5. GroundingMode is a closed 5-variant enum (Constructive,
//    FalsifiabilityCrossFamily, ConformalRefusal, OperationalDeferred,
//    ConservativeFallback) and serde stable.
// 6. select_grounding_mode is deterministic in (claim, ctx) and lands
//    each canonical (claim_type, ctx) pair on its expected mode.

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, ClaimRoutedPayload, SsbActionType, SsbPayload,
    SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::{
    select_grounding_mode, Claim, ClaimType, EvidenceKind, EvidenceRef, GroundingMode,
    OrchContext, OrchPhase, OrchUserRole,
};

/// Floor for D-10: schema must include the ClaimRouted variant.
const D10_SCHEMA_FLOOR: i64 = 77;

#[test]
fn schema_version_includes_d10_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= D10_SCHEMA_FLOOR,
        "schema must include D-10 ClaimRouted variant (got {SCHEMA_VERSION_PUBLIC}, floor {D10_SCHEMA_FLOOR})"
    );
}

#[test]
fn claim_routed_action_type_roundtrip() {
    let s = SsbActionType::ClaimRouted.as_sql_str();
    assert_eq!(s, "claim_routed");
    let decoded =
        ssb_action_type_from_str_public(s).expect("ssb_action_type_from_str_public claim_routed");
    assert_eq!(decoded, SsbActionType::ClaimRouted);
}

#[test]
fn claim_routed_payload_msgpack_roundtrip() {
    let p = ClaimRoutedPayload {
        claim_id: "claim-001".into(),
        claim_type: "factual".into(),
        mode: "constructive".into(),
        ts_ms: 1_700_000_000_000,
        ctx_summary_blake3: "a".repeat(64),
    };
    let payload = SsbPayload::ClaimRouted(p.clone());
    assert_eq!(payload.action_type(), SsbActionType::ClaimRouted);

    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack deserialize");
    assert_eq!(decoded.action_type(), SsbActionType::ClaimRouted);

    if let SsbPayload::ClaimRouted(d) = decoded {
        assert_eq!(d, p);
    } else {
        panic!("expected ClaimRouted payload");
    }
}

#[test]
fn claim_routed_payload_json_roundtrip() {
    let p = ClaimRoutedPayload {
        claim_id: "claim-json".into(),
        claim_type: "computational".into(),
        mode: "operational_deferred".into(),
        ts_ms: 1_700_000_000_500,
        ctx_summary_blake3: "b".repeat(64),
    };
    let json = serde_json::to_string(&p).expect("json serialize");
    assert!(json.contains("\"claim_id\""));
    assert!(json.contains("\"claim_type\""));
    assert!(json.contains("\"mode\""));
    assert!(json.contains("\"ts_ms\""));
    assert!(json.contains("\"ctx_summary_blake3\""));
    let p2: ClaimRoutedPayload = serde_json::from_str(&json).expect("json deserialize");
    assert_eq!(p2, p);
}

#[test]
fn grounding_mode_is_closed_5_variant_enum() {
    let all = GroundingMode::all();
    assert_eq!(all.len(), 5, "GroundingMode must be a 5-variant closed enum");
    // Ensure as_str / serde tags match.
    let labels: Vec<&'static str> = all.iter().map(|m| m.as_str()).collect();
    assert!(labels.contains(&"constructive"));
    assert!(labels.contains(&"falsifiability_cross_family"));
    assert!(labels.contains(&"conformal_refusal"));
    assert!(labels.contains(&"operational_deferred"));
    assert!(labels.contains(&"conservative_fallback"));
}

#[test]
fn grounding_mode_serde_tag_matches_as_str() {
    for m in GroundingMode::all() {
        let json = serde_json::to_string(&m).expect("serialize");
        let stripped = json.trim_matches('"');
        assert_eq!(stripped, m.as_str(), "serde/as_str mismatch for {m:?}");
    }
}

#[test]
fn select_grounding_mode_is_deterministic_per_input() {
    // 5 canonical claims, one per expected mode. Mirrors the
    // observable_effect smoke and gives the dashboard a fixed table
    // to verify against if the predicate ever drifts.

    fn mk_claim(claim_type: ClaimType, with_evidence: bool, id: &str) -> Claim {
        let evidence = if with_evidence {
            vec![EvidenceRef::from_content(
                EvidenceKind::Corpus,
                "src/main.rs:1-10",
                b"fn main() {}",
            )]
        } else {
            vec![]
        };
        Claim {
            claim_id: id.into(),
            text: "test".into(),
            claim_type,
            evidence,
            ts_ms: 1_700_000_000_000,
        }
    }

    use elai_orchestrator::verification_result::VerifierKind;

    let mut ctx_constructive = OrchContext::minimal(OrchUserRole::Executor, OrchPhase::Executing);
    ctx_constructive.available_verifiers =
        vec![VerifierKind::Deterministic, VerifierKind::Intrinsic];

    let mut ctx_cross_family = OrchContext::minimal(OrchUserRole::Executor, OrchPhase::Executing);
    ctx_cross_family.available_verifiers = vec![VerifierKind::Intrinsic, VerifierKind::Ppm];

    let mut ctx_refusal = OrchContext::minimal(OrchUserRole::Executor, OrchPhase::Executing);
    ctx_refusal.available_verifiers = vec![VerifierKind::Intrinsic];
    ctx_refusal.conformal_calibration_ready = true;

    let mut ctx_operational = OrchContext::minimal(OrchUserRole::Executor, OrchPhase::Executing);
    ctx_operational.available_verifiers = vec![VerifierKind::Deterministic];

    let ctx_fallback = OrchContext::minimal(OrchUserRole::Executor, OrchPhase::Executing);

    let rows: [(ClaimType, bool, &OrchContext, GroundingMode); 5] = [
        (
            ClaimType::Factual,
            true,
            &ctx_constructive,
            GroundingMode::Constructive,
        ),
        (
            ClaimType::Speculative,
            false,
            &ctx_cross_family,
            GroundingMode::FalsifiabilityCrossFamily,
        ),
        (
            ClaimType::Speculative,
            false,
            &ctx_refusal,
            GroundingMode::ConformalRefusal,
        ),
        (
            ClaimType::Procedural,
            false,
            &ctx_operational,
            GroundingMode::OperationalDeferred,
        ),
        (
            ClaimType::Normative,
            true,
            &ctx_fallback,
            GroundingMode::ConservativeFallback,
        ),
    ];

    for (i, (claim_type, evidence, ctx, expected)) in rows.iter().enumerate() {
        let claim = mk_claim(*claim_type, *evidence, &format!("c-{i}"));
        let mode_first = select_grounding_mode(&claim, ctx);
        let mode_second = select_grounding_mode(&claim, ctx);
        assert_eq!(mode_first, mode_second, "predicate must be deterministic");
        assert_eq!(
            mode_first, *expected,
            "row {i} ({claim_type:?}) expected {:?}, got {:?}",
            expected, mode_first
        );
    }
}
