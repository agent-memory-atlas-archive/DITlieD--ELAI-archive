//! E-38 (SCHEMA_VERSION 175) -- AiRoadmapSignalEvent SSB contract.
//!
//! Asserts the Pattern-2 single-variant + 5-arm RoadmapSignalKind
//! discriminator wires up correctly across:
//!   - SsbActionType enum (variant + as_sql_str + from_str)
//!   - SsbPayload variant + action_type discriminant mapping
//!   - RoadmapSignalKind 5-arm coverage with snake_case wire labels
//!   - SCHEMA_VERSION bumped 174 -> 175 (E-31 head + 1)
//!   - msgpack round-trip stability for each of the 5 kinds
//!
//! Companion to the producer-side compliance gate tests in
//! crates/orchestrator/src/compliance.rs (unit tests inside that module
//! exercise the verdict logic). The compliance pre-gate end-to-end FSM
//! integration test lives in
//! `e38_compliance_pre_gate_integration_contract.rs`.
//!
//! See workflow/reasoning/E-38-decisions.md.

use elai_orchestrator::ssb::{
    AiRoadmapSignalEventPayload, COMPLIANCE_BIT_AUDIT_TRAIL_LIVE,
    COMPLIANCE_BIT_CLOCK_PROVIDER, COMPLIANCE_BIT_DETERMINISTIC_SEED,
    COMPLIANCE_BIT_TEMPLATE_HASH_CHAIN, ELAI_AI_ROADMAP_SIGNALS_ENABLE_ENV,
    MRCR_UNVERIFIED_CONTEXT_THRESHOLD, POSTURE_DOC_STALE_DAYS, RoadmapSignalKind,
    SCHEMA_VERSION_PUBLIC, SsbActionType, SsbPayload, ssb_action_type_from_str_public,
};

#[test]
fn schema_version_bumped_to_175_for_e38() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 175,
        "schema only drifts forward; this contract was written at version 175. \
         E-38 lands at SCHEMA_VERSION >= 175 (174 = E-31 CodingModelLeaderboardEvent + 1 for E-38 AiRoadmapSignalEvent)"
    );
}

#[test]
fn ai_roadmap_signal_event_action_type_round_trips() {
    assert_eq!(
        SsbActionType::AiRoadmapSignalEvent.as_sql_str(),
        "ai_roadmap_signal_event"
    );
    let parsed = ssb_action_type_from_str_public("ai_roadmap_signal_event")
        .expect("ai_roadmap_signal_event must decode");
    assert_eq!(parsed, SsbActionType::AiRoadmapSignalEvent);
}

#[test]
fn roadmap_signal_kind_covers_five_arms_with_snake_case_labels() {
    assert_eq!(RoadmapSignalKind::ComplianceBlocked.as_str(), "compliance_blocked");
    assert_eq!(RoadmapSignalKind::ComplianceVerified.as_str(), "compliance_verified");
    assert_eq!(
        RoadmapSignalKind::McpTransportVersionLogged.as_str(),
        "mcp_transport_version_logged"
    );
    assert_eq!(RoadmapSignalKind::MrcrUnverified.as_str(), "mrcr_unverified");
    assert_eq!(RoadmapSignalKind::PostureDocStale.as_str(), "posture_doc_stale");
}

#[test]
fn env_gate_constant_matches_canonical_name() {
    assert_eq!(ELAI_AI_ROADMAP_SIGNALS_ENABLE_ENV, "ELAI_AI_ROADMAP_SIGNALS_ENABLE");
}

#[test]
fn compliance_bitmask_constants_are_distinct_powers_of_two() {
    assert_eq!(COMPLIANCE_BIT_CLOCK_PROVIDER, 0x1);
    assert_eq!(COMPLIANCE_BIT_DETERMINISTIC_SEED, 0x2);
    assert_eq!(COMPLIANCE_BIT_TEMPLATE_HASH_CHAIN, 0x4);
    assert_eq!(COMPLIANCE_BIT_AUDIT_TRAIL_LIVE, 0x8);
    let all = COMPLIANCE_BIT_CLOCK_PROVIDER
        | COMPLIANCE_BIT_DETERMINISTIC_SEED
        | COMPLIANCE_BIT_TEMPLATE_HASH_CHAIN
        | COMPLIANCE_BIT_AUDIT_TRAIL_LIVE;
    assert_eq!(all, 0xF, "all four bits OR to 0xF (no collision)");
}

#[test]
fn threshold_constants_match_plan_acceptance() {
    assert_eq!(
        POSTURE_DOC_STALE_DAYS, 90,
        "plan task 10 + plan acceptance posture_doc_stale check uses a 90-day refresh window"
    );
    assert_eq!(
        MRCR_UNVERIFIED_CONTEXT_THRESHOLD, 100_000,
        "plan task 4 deterministic-check threshold is 100K declared context tokens"
    );
}

#[test]
fn ssb_payload_action_type_discriminant_maps_correctly() {
    let payload = SsbPayload::AiRoadmapSignalEvent(sample_payload(RoadmapSignalKind::ComplianceBlocked));
    assert_eq!(payload.action_type(), SsbActionType::AiRoadmapSignalEvent);
}

#[test]
fn msgpack_round_trip_all_five_kinds() {
    for kind in [
        RoadmapSignalKind::ComplianceBlocked,
        RoadmapSignalKind::ComplianceVerified,
        RoadmapSignalKind::McpTransportVersionLogged,
        RoadmapSignalKind::MrcrUnverified,
        RoadmapSignalKind::PostureDocStale,
    ] {
        let payload = SsbPayload::AiRoadmapSignalEvent(sample_payload(kind));
        let bytes = rmp_serde::to_vec_named(&payload)
            .unwrap_or_else(|e| panic!("kind {kind:?}: msgpack encode failed: {e}"));
        let decoded: SsbPayload = rmp_serde::from_slice(&bytes)
            .unwrap_or_else(|e| panic!("kind {kind:?}: msgpack decode failed: {e}"));
        assert_eq!(payload, decoded, "round-trip mismatch for kind {kind:?}");
    }
}

#[test]
fn unknown_kind_label_does_not_parse_as_action_type() {
    // Forward-compat guard: future RoadmapSignalKind extensions on the
    // PAYLOAD side must NOT introduce new SsbActionType strings.
    assert!(ssb_action_type_from_str_public("compliance_blocked").is_err());
    assert!(ssb_action_type_from_str_public("mrcr_unverified").is_err());
    // Only the umbrella action_type label parses.
    assert!(ssb_action_type_from_str_public("ai_roadmap_signal_event").is_ok());
}

fn sample_payload(kind: RoadmapSignalKind) -> AiRoadmapSignalEventPayload {
    let base = AiRoadmapSignalEventPayload {
        kind,
        plan_session_id: "s-contract".to_string(),
        assertion_label: String::new(),
        hint: String::new(),
        assertions_checked: 0,
        mcp_transport_current: String::new(),
        mcp_transport_target: String::new(),
        backend_name: String::new(),
        declared_context_tokens: 0,
        mrcr_score_label: String::new(),
        posture_doc_path: String::new(),
        posture_doc_mtime_ms: 0,
        age_days: 0,
        details_json: serde_json::json!({ "caller_site": "e38_contract" }),
        ts_ms: 1_700_000_175_001,
    };
    match kind {
        RoadmapSignalKind::ComplianceBlocked => AiRoadmapSignalEventPayload {
            assertion_label: "clock_provider_intact".to_string(),
            hint: "inject ClockProvider".to_string(),
            ..base
        },
        RoadmapSignalKind::ComplianceVerified => AiRoadmapSignalEventPayload {
            assertions_checked: COMPLIANCE_BIT_CLOCK_PROVIDER | COMPLIANCE_BIT_AUDIT_TRAIL_LIVE,
            ..base
        },
        RoadmapSignalKind::McpTransportVersionLogged => AiRoadmapSignalEventPayload {
            mcp_transport_current: "streamable_http".to_string(),
            mcp_transport_target: "stateless+tasks+registry-q4-2026+saml-oidc-pkce-q2-2026"
                .to_string(),
            ..base
        },
        RoadmapSignalKind::MrcrUnverified => AiRoadmapSignalEventPayload {
            backend_name: "claude-opus-4-7".to_string(),
            declared_context_tokens: 1_000_000,
            mrcr_score_label: "vendor-claim-only".to_string(),
            ..base
        },
        RoadmapSignalKind::PostureDocStale => AiRoadmapSignalEventPayload {
            posture_doc_path: "docs/strategy/ELAI-2026-POSTURE.md".to_string(),
            posture_doc_mtime_ms: 1_700_000_000_000,
            age_days: 120,
            ..base
        },
    }
}
