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


use std::sync::Arc;
use tempfile::TempDir;
use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::ssb::{Ssb, SsbConfig};
use elai_orchestrator::trial_ledger::TrialLedger;
use elai_orchestrator::routing::{
    artifact_config_hash, configured_artifact_path, register_router_promotion,
    resolve_router_artifact, RouterArtifactVerdict,
};

fn router_test_ssb(dir: &TempDir) -> Arc<Ssb> {
    Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: dir.path().join("ssb.db"),
                pool_size: Some(2),
                project_id: "i25-router".into(),
                session_id: "i25-router-session".into(),
            },
            Arc::new(OsClock),
        )
        .expect("ssb"),
    )
}

#[test]
fn artifact_is_blocked_without_ledger_win() {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("router_model.toml");
    let bytes = br#"[model]
kind = "logistic"
"#;
    std::fs::write(&path, bytes).expect("artifact");
    let ledger = TrialLedger::open(&dir.path().join("ssb.db"), "i25").expect("ledger");
    match resolve_router_artifact(&ledger, &path) {
        RouterArtifactVerdict::PromotionBlocked { config_hash, reason } => {
            assert_eq!(config_hash, artifact_config_hash(bytes));
            assert_eq!(reason, "no_ledger_win");
        }
        RouterArtifactVerdict::Loaded { path, .. } => panic!("unblocked path: {path:?}"),
        other => panic!("unexpected verdict: {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifact_loads_only_after_promotion_registration() {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("router_model.toml");
    let bytes = br#"[model]
kind = "logistic"
"#;
    std::fs::write(&path, bytes).expect("artifact");
    let ssb = router_test_ssb(&dir);
    let ledger = TrialLedger::open(&dir.path().join("ssb.db"), ssb.session_id()).expect("ledger");
    // HARD: must be Arc<dyn ClockProvider>, not Arc<OsClock>
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    register_router_promotion(&ledger, &ssb, &clock, bytes, 17)
        .await
        .expect("promotion registration");
    match resolve_router_artifact(&ledger, &path) {
        RouterArtifactVerdict::Loaded {
            path: loaded,
            config_hash,
        } => {
            assert_eq!(loaded, path);
            assert_eq!(config_hash, artifact_config_hash(bytes));
        }
        other => panic!("expected loaded artifact, got {other:?}"),
    }
}

#[test]
fn artifact_path_is_disabled_when_env_unset() {
    std::env::remove_var("ELAI_ROUTER_ARTIFACT_PATH");
    assert!(configured_artifact_path().is_none());
}
