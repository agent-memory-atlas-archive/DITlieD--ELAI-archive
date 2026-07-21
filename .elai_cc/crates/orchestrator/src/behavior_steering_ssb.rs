//! J-24: BehaviorSteeringEvent SSB emit helper + steering boot flag.
//!
//! Mirrors `prefix_cache_ssb::emit_local_prefix_cache_event`: a missed
//! SSB row is never a correctness failure. When no tokio runtime is
//! available, a transient current-thread runtime is built so the row still
//! lands (do NOT spawn-and-drop).

use std::env;
use std::sync::Arc;

use crate::ssb::{
    system_time_ms_i64, BehaviorSteeringEventPayload, BehaviorSteeringKind, Ssb, SsbPayload,
};

/// Environment variable that enables steering at sidecar boot (J-24 LEG1=A).
pub const ELAI_STEERING_ENABLE_ENV: &str = "ELAI_STEERING_ENABLE";

/// Returns the sidecar boot flag for steering based on `ELAI_STEERING_ENABLE`.
/// When set to `"1"`, returns `"--steering"`; otherwise returns
/// `"--no-steering"` (preserving the default-off behaviour). The Python
/// argparse uses `--steering` (store_true) / `--no-steering` (store_false)
/// with default False, so the affirmative flag is required to enable.
pub fn steering_boot_flag() -> &'static str {
    if env::var(ELAI_STEERING_ENABLE_ENV).ok().as_deref() == Some("1") {
        "--steering"
    } else {
        "--no-steering"
    }
}

/// Emit a `BehaviorSteeringEvent` SSB row.
///
/// Non-fatal: warns on write failure, never propagates. The `agent_id` is
/// `"behavior_steering"`, superstep is 0. The timestamp comes from
/// [`crate::ssb::system_time_ms_i64`].
pub fn emit_behavior_steering_event(
    ssb: &Arc<Ssb>,
    kind: BehaviorSteeringKind,
    details_json: serde_json::Value,
) {
    let ts_ms = system_time_ms_i64();
    let payload = SsbPayload::BehaviorSteeringEvent(BehaviorSteeringEventPayload {
        kind,
        details_json,
        ts_ms,
    });
    let ssb = Arc::clone(ssb);
    let agent_id = "behavior_steering";
    let emit = async move {
        if let Err(e) = ssb.write_payload(agent_id, 0, payload, vec![]).await {
            tracing::warn!(
                behavior_steering.ssb_err = %e,
                "BehaviorSteeringEvent SSB emit failed (steering unaffected)"
            );
        }
    };
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            handle.spawn(emit);
        }
        Err(_) => {
            if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                rt.block_on(emit);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssb::{SsbActionType, SsbConfig, SsbReader};
    use elai_core::clock::SeededClock;
    use tempfile::TempDir;

    #[test]
    fn kind_as_str_roundtrip() {
        assert_eq!(BehaviorSteeringKind::SteeringVectorApplied.as_str(), "steering_vector_applied");
        assert_eq!(BehaviorSteeringKind::SteeringEffectMeasured.as_str(), "steering_effect_measured");
        assert_eq!(BehaviorSteeringKind::ObservationSizePreassessed.as_str(), "observation_size_preassessed");
    }

    #[test]
    fn payload_action_type_maps_correctly() {
        let payload = BehaviorSteeringEventPayload {
            kind: BehaviorSteeringKind::ObservationSizePreassessed,
            details_json: serde_json::json!({"observation_tokens": 4096}),
            ts_ms: 1_700_000_000_000,
        };
        let wrapped = SsbPayload::BehaviorSteeringEvent(payload);
        let action_type = wrapped.action_type();
        assert_eq!(
            action_type.as_sql_str(),
            "behavior_steering_event"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn emit_observation_size_preassessed_row_lands() {
        let dir = TempDir::new().expect("tempdir");
        let config = SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "test-project".into(),
            session_id: "test-session".into(),
        };
        let clock = Arc::new(SeededClock::new()) as Arc<dyn elai_core::clock::ClockProvider>;
        let ssb = Arc::new(Ssb::new(config, clock).expect("ssb init"));

        emit_behavior_steering_event(
            &ssb,
            BehaviorSteeringKind::ObservationSizePreassessed,
            serde_json::json!({"observation_tokens": 4096, "steering_eligible": true}),
        );

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let entries = ssb
            .query_by_type(SsbActionType::BehaviorSteeringEvent(
                BehaviorSteeringKind::ObservationSizePreassessed,
            ))
            .expect("query_by_type");
        assert_eq!(entries.len(), 1, "expected one BehaviorSteeringEvent row");
        let entry = &entries[0];
        assert_eq!(entry.agent_id, "behavior_steering");
        assert_eq!(
            entry.action_type.as_sql_str(),
            "behavior_steering_event"
        );
    }
}
