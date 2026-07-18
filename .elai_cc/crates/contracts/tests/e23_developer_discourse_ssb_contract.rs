// E-23 (SCHEMA_VERSION 171) contract: developer-discourse 2026 trust-
// telemetry plane SSB variant.
//
// Verifies:
// 1. SCHEMA_VERSION_PUBLIC >= 171 (E-23 atomic bump for the single
//    DeveloperDiscourseEvent variant with 3-arm DiscourseEventKind
//    discriminator).
// 2. DeveloperDiscourseEvent round-trips via as_sql_str /
//    ssb_action_type_from_str_public.
// 3. action_type() on SsbPayload::DeveloperDiscourseEvent returns the
//    matching SsbActionType.
// 4. DeveloperDiscourseEventPayload serializes via msgpack (rmp_serde)
//    without loss for all 3 DiscourseEventKind arms
//    (TrustTelemetryAggregated / RuntimeChoiceBadgeEmitted /
//    ContextStateWarning).
// 5. ELAI_DEVELOPER_DISCOURSE_ENABLE_ENV constant matches the documented
//    env var name (wire-dark gate).
//
// Grounding: E-23 plan body (docs/thefinalglance/E-plan/E-23*.md),
// trust_telemetry.rs module docs, payloads.rs:11670..11757.

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, DeveloperDiscourseEventPayload, DiscourseEventKind,
    SsbActionType, SsbPayload, ELAI_DEVELOPER_DISCOURSE_ENABLE_ENV, SCHEMA_VERSION_PUBLIC,
};

const E23_SCHEMA_FLOOR: i64 = 171;

#[test]
fn schema_version_includes_e23_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= E23_SCHEMA_FLOOR,
        "schema must include E-23 SSB variant (got {SCHEMA_VERSION_PUBLIC}, floor {E23_SCHEMA_FLOOR})"
    );
}

#[test]
fn developer_discourse_event_action_type_roundtrip() {
    let at = SsbActionType::DeveloperDiscourseEvent;
    let s = at.as_sql_str();
    assert_eq!(s, "developer_discourse_event", "as_sql_str mismatch");
    let decoded = ssb_action_type_from_str_public(s).expect("from_str failed");
    assert_eq!(decoded, at, "round-trip failed for {s}");
}

#[test]
fn discourse_event_kind_wire_labels() {
    // 3 kinds per E-23 decisions; wire labels match dashboard mapping.
    assert_eq!(
        DiscourseEventKind::TrustTelemetryAggregated.as_str(),
        "trust_telemetry_aggregated",
    );
    assert_eq!(
        DiscourseEventKind::RuntimeChoiceBadgeEmitted.as_str(),
        "runtime_choice_badge_emitted",
    );
    assert_eq!(
        DiscourseEventKind::ContextStateWarning.as_str(),
        "context_state_warning",
    );
}

#[test]
fn payload_bijection_trust_telemetry_aggregated() {
    let original = DeveloperDiscourseEventPayload {
        kind: DiscourseEventKind::TrustTelemetryAggregated,
        session_id: Some("s-e23".into()),
        tasks_per_hour: 2.5,
        findings_per_task: 0.8,
        revert_rate: 0.1,
        ungrounded_count: 3,
        prev_badge: String::new(),
        next_badge: String::new(),
        token_budget_pct: 0.0,
        threshold_label: String::new(),
        details_json: serde_json::json!({
            "caller_site": "contract_test",
            "stagnation_dist": {"spinning": 2},
        }),
        ts_ms: 1_700_000_171_001,
    };
    let payload = SsbPayload::DeveloperDiscourseEvent(original.clone());
    assert_eq!(payload.action_type(), SsbActionType::DeveloperDiscourseEvent);
    let encoded = rmp_serde::to_vec(&payload).expect("msgpack encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&encoded).expect("msgpack decode");
    match decoded {
        SsbPayload::DeveloperDiscourseEvent(p) => assert_eq!(p, original),
        other => panic!("unexpected variant after round-trip: {other:?}"),
    }
}

#[test]
fn payload_bijection_runtime_choice_badge_emitted() {
    let original = DeveloperDiscourseEventPayload {
        kind: DiscourseEventKind::RuntimeChoiceBadgeEmitted,
        session_id: Some("s-e23-badge".into()),
        tasks_per_hour: 0.0,
        findings_per_task: 0.0,
        revert_rate: 0.0,
        ungrounded_count: 0,
        prev_badge: "local".into(),
        next_badge: "cloud_billed".into(),
        token_budget_pct: 0.0,
        threshold_label: String::new(),
        details_json: serde_json::json!({
            "caller_site": "contract_test",
        }),
        ts_ms: 1_700_000_171_002,
    };
    let payload = SsbPayload::DeveloperDiscourseEvent(original.clone());
    assert_eq!(payload.action_type(), SsbActionType::DeveloperDiscourseEvent);
    let encoded = rmp_serde::to_vec(&payload).expect("msgpack encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&encoded).expect("msgpack decode");
    match decoded {
        SsbPayload::DeveloperDiscourseEvent(p) => assert_eq!(p, original),
        other => panic!("unexpected variant after round-trip: {other:?}"),
    }
}

#[test]
fn payload_bijection_context_state_warning() {
    let original = DeveloperDiscourseEventPayload {
        kind: DiscourseEventKind::ContextStateWarning,
        session_id: None,
        tasks_per_hour: 0.0,
        findings_per_task: 0.0,
        revert_rate: 0.0,
        ungrounded_count: 0,
        prev_badge: String::new(),
        next_badge: String::new(),
        token_budget_pct: 87.5,
        threshold_label: "red".into(),
        details_json: serde_json::json!({
            "caller_site": "contract_test",
        }),
        ts_ms: 1_700_000_171_003,
    };
    let payload = SsbPayload::DeveloperDiscourseEvent(original.clone());
    assert_eq!(payload.action_type(), SsbActionType::DeveloperDiscourseEvent);
    let encoded = rmp_serde::to_vec(&payload).expect("msgpack encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&encoded).expect("msgpack decode");
    match decoded {
        SsbPayload::DeveloperDiscourseEvent(p) => assert_eq!(p, original),
        other => panic!("unexpected variant after round-trip: {other:?}"),
    }
}

#[test]
fn enable_env_const_matches_docs() {
    // The wire-dark env gate must match the name documented in
    // workflow/reasoning/E-23-decisions.md and CLAUDE.md.
    assert_eq!(
        ELAI_DEVELOPER_DISCOURSE_ENABLE_ENV, "ELAI_DEVELOPER_DISCOURSE_ENABLE",
        "env gate name drift -- decisions doc / dashboard / CLI all read this"
    );
}
