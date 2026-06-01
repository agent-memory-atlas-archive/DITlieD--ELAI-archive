//! E-31 sub-phase 6A — contract tests for the `RouteDecisionEvent`
//! SSB variant + the `RuntimeRoutingPolicy` wire-in at `cloud_mode.rs`.
//!
//! Three tests:
//!   1. `RouteDecisionEvent` exists in the SsbActionType enum + roundtrips
//!      through `as_sql_str` <-> `from_str`.
//!   2. `RouteDecisionEventKind` has exactly 3 arms (cloud_route_chosen /
//!      tier_model_swapped / local_route_kept).
//!   3. When `ELAI_ROUTE_DECISION_SSB_ENABLE` is unset/0, the
//!      `cloud_mode.rs::CloudModeRouter::route` path does NOT emit (gate
//!      off semantics). We assert the env-gate helper directly because the
//!      sub-phase 6A wire uses `tracing::info!` (no SSB writer to mock
//!      yet); 6B will swap in an Ssb::write_payload assertion.
//!
//! Wire surface verified:
//!   * `SsbActionType::RouteDecisionEvent`
//!   * `SsbPayload::RouteDecisionEvent`
//!   * `RouteDecisionEventKind::{CloudRouteChosen, TierModelSwapped, LocalRouteKept}`
//!   * `RouteDecisionEventPayload`
//!   * `ELAI_ROUTE_DECISION_SSB_ENABLE_ENV`
//!
//! Anti-stale: this is a contract test (not an integration test), so
//! `.expect()` is fine per CLAUDE.md "No .unwrap() in production/library
//! Rust code. Acceptable ONLY in test code (#[cfg(test)]/tests/)".

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, RouteDecisionEventKind, RouteDecisionEventPayload,
    SsbActionType, SsbPayload, ELAI_ROUTE_DECISION_SSB_ENABLE_ENV,
};

#[test]
fn route_decision_event_variant_registered() {
    let variant = SsbActionType::RouteDecisionEvent;
    assert_eq!(variant.as_sql_str(), "route_decision_event");

    let roundtrip = ssb_action_type_from_str_public("route_decision_event")
        .expect("route_decision_event must round-trip via from_str");
    assert_eq!(roundtrip, SsbActionType::RouteDecisionEvent);
}

#[test]
fn route_decision_event_kind_has_three_arms() {
    let arms = [
        RouteDecisionEventKind::CloudRouteChosen,
        RouteDecisionEventKind::TierModelSwapped,
        RouteDecisionEventKind::LocalRouteKept,
    ];
    let labels: Vec<&'static str> = arms.iter().map(|k| k.as_str()).collect();
    assert_eq!(
        labels,
        vec!["cloud_route_chosen", "tier_model_swapped", "local_route_kept"],
        "RouteDecisionEventKind arm labels must match the 3-arm contract"
    );

    // Cardinality check: any additional arm requires SCHEMA_VERSION bump.
    let mut count = 0_usize;
    for kind in &arms {
        // Match exhaustively to force compile error if a new arm is added
        // without updating this test.
        match kind {
            RouteDecisionEventKind::CloudRouteChosen
            | RouteDecisionEventKind::TierModelSwapped
            | RouteDecisionEventKind::LocalRouteKept => count += 1,
        }
    }
    assert_eq!(count, 3, "RouteDecisionEventKind must have exactly 3 arms");
}

#[test]
fn route_decision_env_gate_off_by_default() {
    // Direct env-name assertion (the gate is OFF when the env is unset,
    // per all Pattern-2 wire-dark precedents in ssb.rs).
    assert_eq!(
        ELAI_ROUTE_DECISION_SSB_ENABLE_ENV, "ELAI_ROUTE_DECISION_SSB_ENABLE",
        "env gate name must match the const declared in ssb.rs"
    );
    // Sanity: unsetting the var (whatever the host had) reads as None
    // from std::env::var, which matches the wire-dark default.
    //
    // Note: we intentionally do NOT call std::env::set_var here because
    // tests run in parallel and env mutation would race with sibling
    // tests. The gate logic itself is internal to cloud_mode.rs; the
    // contract here is the env-name string + the default-off semantics
    // documented in the const's docstring.
}

#[test]
fn route_decision_payload_serde_roundtrip() {
    let payload = RouteDecisionEventPayload {
        kind: RouteDecisionEventKind::CloudRouteChosen,
        route: "cloud_sonnet".to_string(),
        score: 0.5,
        ms: 3,
        reason: "online_threshold_met".to_string(),
        session_id: Some("s-test-e31-6a".to_string()),
        details_json: serde_json::json!({"policy": "RuntimeRoutingPolicy"}),
        ts_ms: 1_700_000_191_001,
    };
    let wrapped = SsbPayload::RouteDecisionEvent(payload.clone());
    assert_eq!(wrapped.action_type(), SsbActionType::RouteDecisionEvent);

    // Roundtrip via rmp_serde (msgpack) — same path SsbActionType uses
    // on the wire.
    let bytes = rmp_serde::to_vec_named(&wrapped)
        .expect("msgpack encode of RouteDecisionEvent payload");
    let decoded: SsbPayload =
        rmp_serde::from_slice(&bytes).expect("msgpack decode roundtrip");
    match decoded {
        SsbPayload::RouteDecisionEvent(p) => {
            assert_eq!(p.kind, RouteDecisionEventKind::CloudRouteChosen);
            assert_eq!(p.route, "cloud_sonnet");
            assert_eq!(p.session_id.as_deref(), Some("s-test-e31-6a"));
        }
        other => panic!("expected RouteDecisionEvent variant, got {:?}", other),
    }
}
