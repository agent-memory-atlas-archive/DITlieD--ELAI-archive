//! E-17 SP2 (SCHEMA_VERSION 97) — contract tests for the 4 new SsbActionType
//! variants. Verifies:
//!   - SCHEMA_VERSION public constant is 97.
//!   - Each new variant has a stable snake_case sql identifier.
//!   - as_sql_str + from_str are symmetric round-trips.
//!   - SsbPayload variants exist and round-trip through serde.

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, DeepPoolCappedPayload, HydeSkippedCodeQueryPayload,
    RrfComponentWeightsPayload, RrfQualityGateFiredPayload, SsbActionType, SsbPayload,
    SCHEMA_VERSION_PUBLIC,
};

#[test]
fn schema_version_is_97() {
    assert_eq!(SCHEMA_VERSION_PUBLIC, 97, "E-17 SP2 bumped 96 -> 97");
}

#[test]
fn new_action_type_sql_identifiers_are_stable() {
    assert_eq!(SsbActionType::DeepPoolCapped.as_sql_str(), "deep_pool_capped");
    assert_eq!(
        SsbActionType::RrfQualityGateFired.as_sql_str(),
        "rrf_quality_gate_fired"
    );
    assert_eq!(
        SsbActionType::HydeSkippedCodeQuery.as_sql_str(),
        "hyde_skipped_code_query"
    );
    assert_eq!(
        SsbActionType::RrfComponentWeights.as_sql_str(),
        "rrf_component_weights"
    );
}

#[test]
fn from_str_resolves_new_variants() {
    assert_eq!(
        ssb_action_type_from_str_public("deep_pool_capped").expect("known"),
        SsbActionType::DeepPoolCapped,
    );
    assert_eq!(
        ssb_action_type_from_str_public("rrf_quality_gate_fired").expect("known"),
        SsbActionType::RrfQualityGateFired,
    );
    assert_eq!(
        ssb_action_type_from_str_public("hyde_skipped_code_query").expect("known"),
        SsbActionType::HydeSkippedCodeQuery,
    );
    assert_eq!(
        ssb_action_type_from_str_public("rrf_component_weights").expect("known"),
        SsbActionType::RrfComponentWeights,
    );
}

#[test]
fn payload_action_type_is_consistent_with_enum() {
    let p1 = SsbPayload::DeepPoolCapped(DeepPoolCappedPayload {
        capped_count: 150,
        max_allowed: 100,
        stage_count: 5,
    });
    let p2 = SsbPayload::RrfQualityGateFired(RrfQualityGateFiredPayload {
        dropped_count: 12,
        floor: 0.005,
        total_before: 80,
    });
    let p3 = SsbPayload::HydeSkippedCodeQuery(HydeSkippedCodeQueryPayload {
        query_class: "code_symbol".into(),
        word_count: 3,
        code_signal_present: true,
    });
    let p4 = SsbPayload::RrfComponentWeights(RrfComponentWeightsPayload {
        bm25_weight: Some(1.0),
        code_vec_weight: Some(1.0),
        text_vec_weight: Some(0.8),
    });
    assert_eq!(p1.action_type(), SsbActionType::DeepPoolCapped);
    assert_eq!(p2.action_type(), SsbActionType::RrfQualityGateFired);
    assert_eq!(p3.action_type(), SsbActionType::HydeSkippedCodeQuery);
    assert_eq!(p4.action_type(), SsbActionType::RrfComponentWeights);
}

#[test]
fn payload_serde_roundtrip() {
    let payloads: Vec<SsbPayload> = vec![
        SsbPayload::DeepPoolCapped(DeepPoolCappedPayload {
            capped_count: 150,
            max_allowed: 100,
            stage_count: 5,
        }),
        SsbPayload::RrfQualityGateFired(RrfQualityGateFiredPayload {
            dropped_count: 12,
            floor: 0.005,
            total_before: 80,
        }),
        SsbPayload::HydeSkippedCodeQuery(HydeSkippedCodeQueryPayload {
            query_class: "code_symbol".into(),
            word_count: 3,
            code_signal_present: true,
        }),
        SsbPayload::RrfComponentWeights(RrfComponentWeightsPayload {
            bm25_weight: Some(1.0),
            code_vec_weight: None,
            text_vec_weight: Some(0.8),
        }),
    ];
    for p in payloads {
        let json = serde_json::to_string(&p).expect("serialize");
        let back: SsbPayload = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(p.action_type(), back.action_type());
    }
}
