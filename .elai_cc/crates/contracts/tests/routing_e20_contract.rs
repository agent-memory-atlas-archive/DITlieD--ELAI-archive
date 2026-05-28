//! E-20 (SCHEMA_VERSION 180) routing contract test.
//!
//! Asserts:
//!   1. `SsbActionType::LocalCloudRoutingEvent` round-trips through the
//!      string-to-enum map.
//!   2. All 9 `RoutingEventKind` arms have unique, snake_case wire labels.
//!   3. `LocalCloudRoutingEventPayload` serdes round-trips for every kind
//!      with a representative `details_json`.
//!   4. `PerRoleRuntime` `priority` + `task_shape_hint` Default-coverage
//!      survives serde round-trip with all 16 AgentRole variants
//!      populated.
//!   5. `RouteBlockedPayload.reason = "privacy_block"` is a valid value
//!      (no enum cascade required for the PrivacyBlock fire-point).

use elai_ipc_types::AgentRole;
use elai_orchestrator::per_role_runtime::{PerRoleRuntime, Priority, TaskShape};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, LocalCloudRoutingEventPayload, RouteBlockedPayload,
    RoutingEventKind, SsbActionType, SCHEMA_VERSION_PUBLIC,
};

#[test]
fn schema_version_floor_180() {
    // E-20 introduced SCHEMA 180. E-21 bumped to 181 (PeerSystemsEvent);
    // E-36 bumped to 182 (DocumentVisionRagEvent). Keep this assertion as
    // a FLOOR (>= 180) rather than an exact pin so future single-variant
    // Pattern-2 ships do not require touching this test (and so the E-20
    // contract surface stays orthogonal to subsequent SCHEMA bumps).
    assert!(
        SCHEMA_VERSION_PUBLIC >= 180,
        "SCHEMA_VERSION_PUBLIC regressed below E-20 floor 180; got {SCHEMA_VERSION_PUBLIC}"
    );
}

#[test]
fn action_type_string_round_trips() {
    assert_eq!(
        SsbActionType::LocalCloudRoutingEvent.as_sql_str(),
        "local_cloud_routing_event"
    );
    let parsed = ssb_action_type_from_str_public("local_cloud_routing_event")
        .expect("from_str must succeed");
    assert_eq!(parsed, SsbActionType::LocalCloudRoutingEvent);
}

#[test]
fn routing_event_kind_labels_unique_and_snake_case() {
    use RoutingEventKind::*;
    let kinds = [
        ConfidenceEmitted,
        HybridEscalation,
        DifficultyClassified,
        PrivacyBlocked,
        OutputFiltered,
        BurnAlert,
        ReasoningMode,
        MajorityVote,
        RolePriorityChosen,
    ];
    let labels: Vec<&str> = kinds.iter().map(|k| k.as_str()).collect();
    // 9 unique labels.
    let mut sorted = labels.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), 9);
    // All snake_case (ASCII lowercase + digits + underscore).
    for label in &labels {
        assert!(
            label
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            "label {label:?} is not snake_case"
        );
    }
}

#[test]
fn payload_round_trips_for_every_kind() {
    let kinds = [
        RoutingEventKind::ConfidenceEmitted,
        RoutingEventKind::HybridEscalation,
        RoutingEventKind::DifficultyClassified,
        RoutingEventKind::PrivacyBlocked,
        RoutingEventKind::OutputFiltered,
        RoutingEventKind::BurnAlert,
        RoutingEventKind::ReasoningMode,
        RoutingEventKind::MajorityVote,
        RoutingEventKind::RolePriorityChosen,
    ];
    for kind in kinds {
        let payload = LocalCloudRoutingEventPayload {
            kind,
            session_id: Some("s-test".into()),
            task_id: Some("t-test".into()),
            details_json: serde_json::json!({"sample": "value"}),
            ts_ms: 12345,
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        let decoded: LocalCloudRoutingEventPayload =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.kind, kind);
        assert_eq!(decoded.session_id, payload.session_id);
        assert_eq!(decoded.task_id, payload.task_id);
        assert_eq!(decoded.ts_ms, payload.ts_ms);
    }
}

#[test]
fn per_role_runtime_priority_full_coverage_survives_serde() {
    let original = PerRoleRuntime::default();
    let json = serde_json::to_string(&original).expect("serialize");
    let decoded: PerRoleRuntime = serde_json::from_str(&json).expect("deserialize");
    let all_roles = [
        AgentRole::Executor,
        AgentRole::Auditor,
        AgentRole::Researcher,
        AgentRole::Designer,
        AgentRole::PromptEngineer,
        AgentRole::ContextCompiler,
        AgentRole::MemoryWriter,
        AgentRole::MediaValidator,
        AgentRole::IdeaFramer,
        AgentRole::IdeaCritic,
        AgentRole::AuditorConcepts,
        AgentRole::AuditorPhases,
        AgentRole::AuditorSteps,
        AgentRole::AuditorBabySteps,
        AgentRole::AuditorBabyStepDetail,
        AgentRole::InjectionClassifier,
    ];
    for role in all_roles {
        assert!(
            decoded.priority.contains_key(&role),
            "priority missing for {role:?}"
        );
        assert!(
            decoded.task_shape_hint.contains_key(&role),
            "task_shape_hint missing for {role:?}"
        );
    }
}

#[test]
fn priority_serde_labels_are_snake_case() {
    let json_cost = serde_json::to_string(&Priority::Cost).expect("serialize cost");
    let json_quality = serde_json::to_string(&Priority::Quality).expect("serialize quality");
    assert_eq!(json_cost, "\"cost\"");
    assert_eq!(json_quality, "\"quality\"");
}

#[test]
fn task_shape_serde_labels_are_snake_case() {
    let json_arch = serde_json::to_string(&TaskShape::Architecture)
        .expect("serialize architecture");
    assert_eq!(json_arch, "\"architecture\"");
}

#[test]
fn route_blocked_payload_accepts_privacy_block_reason() {
    // PrivacyBlock composes with the pre-existing RouteBlocked variant
    // via a free-form reason string -- no enum cascade required.
    let payload = RouteBlockedPayload {
        reason: "privacy_block".into(),
        spent_usd: 0.0,
        cap_usd: 0.0,
        cap_period: "policy".into(),
        baby_step_id: 42,
        ts_ms: 1000,
    };
    let json = serde_json::to_string(&payload).expect("serialize");
    let decoded: RouteBlockedPayload = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded.reason, "privacy_block");
    assert_eq!(decoded.cap_period, "policy");
}
