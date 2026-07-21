// J-24 contract: BehaviorSteeringEvent SSB variant (Pattern-2, 3-arm Kind).
//
// Verifies:
// 1. SCHEMA_VERSION_PUBLIC >= 287 (J-24 adds 1 variant to the 286 baseline).
// 2. BehaviorSteeringEvent action_type round-trips via as_sql_str /
//    ssb_action_type_from_str_public.
// 3. Each of the 3 BehaviorSteeringKind arms produces a valid
//    BehaviorSteeringEventPayload that serializes via msgpack + JSON
//    without loss (round-trip equality).
// 4. SsbPayload::BehaviorSteeringEvent maps to
//    SsbActionType::BehaviorSteeringEvent (bijection).
// 5. TOTAL_VARIANTS == 596 (SSB_ACTION_TYPES.len()).
// 6. NEGATIVE: ToolSchemaDeferred reuses McpToolDeferred action_type string
//    (no new SSB arm was added for it per LEG3=B).

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, BehaviorSteeringEventPayload, BehaviorSteeringKind,
    SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC,
};
use serde_json::json;

const J24_SCHEMA_FLOOR: i64 = 287;

#[test]
fn schema_version_includes_j24_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= J24_SCHEMA_FLOOR,
        "schema must include J-24 BehaviorSteeringEvent (got {SCHEMA_VERSION_PUBLIC}, floor {J24_SCHEMA_FLOOR})"
    );
}

#[test]
fn behavior_steering_event_action_type_roundtrip() {
    let at = SsbActionType::BehaviorSteeringEvent(BehaviorSteeringKind::ObservationSizePreassessed);
    let s = at.as_sql_str();
    assert_eq!(s, "behavior_steering_event");
    let decoded = ssb_action_type_from_str_public(s).expect("round-trip");
    assert_eq!(decoded, at);
}

#[test]
fn total_variants_is_596() {
    let total = elai_orchestrator::ssb::SSB_ACTION_TYPES.len();
    assert_eq!(
        total, 596,
        "TOTAL_VARIANTS must be 596 after J-24 (got {total})"
    );
}

fn roundtrip_payload(kind: BehaviorSteeringKind, details: serde_json::Value) {
    let payload = BehaviorSteeringEventPayload {
        kind,
        details_json: details,
        ts_ms: 1_700_000_000_000,
    };

    let msgpack_bytes = rmp_serde::to_vec(&payload).expect("msgpack serialize");
    let decoded: BehaviorSteeringEventPayload =
        rmp_serde::from_slice(&msgpack_bytes).expect("msgpack deserialize");
    assert_eq!(decoded, payload, "msgpack round-trip failed for {kind:?}");

    let json_str = serde_json::to_string(&payload).expect("json serialize");
    let decoded_json: BehaviorSteeringEventPayload =
        serde_json::from_str(&json_str).expect("json deserialize");
    assert_eq!(decoded_json, payload, "json round-trip failed for {kind:?}");

    let wrapped = SsbPayload::BehaviorSteeringEvent(payload);
    let action_type = wrapped.action_type();
    assert_eq!(
        action_type,
        SsbActionType::BehaviorSteeringEvent(kind),
        "payload->action_type bijection failed for {kind:?}"
    );
}

#[test]
fn fixture_steering_vector_applied() {
    roundtrip_payload(
        BehaviorSteeringKind::SteeringVectorApplied,
        json!({
            "steering_strength": 0.75,
            "vector_dim": 4096,
            "layer": 12
        }),
    );
}

#[test]
fn fixture_steering_effect_measured() {
    roundtrip_payload(
        BehaviorSteeringKind::SteeringEffectMeasured,
        json!({
            "effect_delta": -0.12,
            "baseline_score": 0.85,
            "steered_score": 0.73
        }),
    );
}

#[test]
fn fixture_observation_size_preassessed() {
    roundtrip_payload(
        BehaviorSteeringKind::ObservationSizePreassessed,
        json!({
            "observation_tokens": 4096,
            "steering_eligible": true
        }),
    );
}

#[test]
fn negative_tool_schema_deferred_uses_mcp_tool_deferred_string() {
    let mcp_str = SsbActionType::McpToolDeferred.as_sql_str();
    assert_eq!(
        mcp_str, "mcp_tool_deferred",
        "ToolSchemaDeferred must reuse McpToolDeferred action_type string (LEG3=B)"
    );
    let decoded = ssb_action_type_from_str_public("mcp_tool_deferred").expect("decode");
    assert_eq!(
        decoded,
        SsbActionType::McpToolDeferred,
        "mcp_tool_deferred must decode to McpToolDeferred, not a new arm"
    );
}
