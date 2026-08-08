//! I-25 Slice 1 contract: deterministic cost profiling and route-event shape.
//!
//! Env-gate checks live in one test (serial steps) so parallel harness runs
//! cannot race on process-global ELAI_COST_PROFILER_ENABLE.

use elai_orchestrator::routing::{cost_profiler_enabled, CostProfileVerdict, CostProfiler};
use elai_orchestrator::ssb::{RouteDecisionEventKind, RouteDecisionEventPayload, SsbPayload};

#[test]
fn expensive_shape_and_event_contract() {
    // Pure classifier — no env dependency (avoids parallel env races).
    let verdict =
        CostProfiler::new().classify("rewrite entire multi-file refactor across the crate");
    let (fingerprint, feature_vec) = match verdict {
        CostProfileVerdict::Expensive {
            fingerprint,
            feature_vec,
        } => (fingerprint, feature_vec),
        CostProfileVerdict::Cheap => panic!("expensive fixture classified cheap"),
    };
    let reason = "cost_profile_expensive_shape";
    let details = serde_json::json!({
        "reason": reason,
        "predicted_lane": "cloud",
        "feature_vec": feature_vec,
        "fingerprint": fingerprint,
    });
    let event = SsbPayload::RouteDecisionEvent(RouteDecisionEventPayload {
        kind: RouteDecisionEventKind::CloudRouteChosen,
        route: "cloud".into(),
        score: 1.0,
        ms: 0,
        reason: reason.into(),
        session_id: None,
        details_json: details,
        ts_ms: 0,
    });
    let bytes = rmp_serde::to_vec_named(&event).expect("event serialization");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("event deserialization");
    match decoded {
        SsbPayload::RouteDecisionEvent(payload) => {
            assert_eq!(payload.reason, reason);
            assert_eq!(
                payload.details_json.get("reason").and_then(|v| v.as_str()),
                Some(reason)
            );
        }
        _ => panic!("wrong event variant"),
    }
}

#[test]
fn cheap_classify_and_env_gate_serial() {
    // Sequential env steps inside one test — no cross-test process env races.
    // SAFETY: single-threaded critical section of this test body only.
    std::env::set_var("ELAI_COST_PROFILER_ENABLE", "true");
    assert!(cost_profiler_enabled());
    std::env::remove_var("ELAI_COST_PROFILER_ENABLE");
    assert!(!cost_profiler_enabled());
    assert_eq!(
        CostProfiler::new().classify("fix one typo"),
        CostProfileVerdict::Cheap
    );
}
