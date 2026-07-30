//! J-12: HarnessInterventionEvent SSB emit helper.
//!
//! Mirrors `behavior_steering_ssb::emit_behavior_steering_event`: a missed
//! SSB row is never a correctness failure. When no tokio runtime is
//! available, a transient current-thread runtime is built so the row still
//! lands (do NOT spawn-and-drop).

use std::sync::Arc;

use crate::ssb::{
    system_time_ms_i64, HarnessInterventionEventPayload, HarnessInterventionKind, Ssb, SsbPayload,
};

/// Emit a `HarnessInterventionEvent` SSB row.
///
/// Non-fatal: warns on write failure, never propagates. The `agent_id` is
/// `"harness_intervention"`, superstep is 0. The timestamp comes from
/// [`crate::ssb::system_time_ms_i64`].
pub fn emit_harness_intervention_event(
    ssb: &Arc<Ssb>,
    kind: HarnessInterventionKind,
    details_json: serde_json::Value,
) {
    let ts_ms = system_time_ms_i64();
    let payload = SsbPayload::HarnessInterventionEvent(HarnessInterventionEventPayload {
        kind,
        details_json,
        ts_ms,
    });
    let ssb = Arc::clone(ssb);
    let agent_id = "harness_intervention";
    let emit = async move {
        if let Err(e) = ssb.write_payload(agent_id, 0, payload, vec![]).await {
            tracing::warn!(
                harness_intervention.ssb_err = %e,
                "HarnessInterventionEvent SSB emit failed (intervention path unaffected)"
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
        assert_eq!(HarnessInterventionKind::ContractInjected.as_str(), "contract_injected");
        assert_eq!(HarnessInterventionKind::ActionGateBlocked.as_str(), "action_gate_blocked");
        assert_eq!(HarnessInterventionKind::ResultAnnotated.as_str(), "result_annotated");
        assert_eq!(HarnessInterventionKind::SkillRetrieved.as_str(), "skill_retrieved");
    }

    #[test]
    fn payload_action_type_maps_correctly() {
        let payload = HarnessInterventionEventPayload {
            kind: HarnessInterventionKind::ActionGateBlocked,
            details_json: serde_json::json!({"rule_id": "r1"}),
            ts_ms: 1_700_000_000_000,
        };
        let wrapped = SsbPayload::HarnessInterventionEvent(payload);
        let action_type = wrapped.action_type();
        assert_eq!(action_type.as_sql_str(), "harness_intervention_event");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn emit_action_gate_blocked_row_lands() {
        let dir = TempDir::new().expect("tempdir");
        let config = SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "test-project".into(),
            session_id: "test-session".into(),
        };
        let clock = Arc::new(SeededClock::new()) as Arc<dyn elai_core::clock::ClockProvider>;
        let ssb = Arc::new(Ssb::new(config, clock).expect("ssb init"));

        emit_harness_intervention_event(
            &ssb,
            HarnessInterventionKind::ActionGateBlocked,
            serde_json::json!({"rule_id": "r1", "message": "missing path", "tool": "write_file"}),
        );

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let entries = ssb
            .query_by_type(SsbActionType::HarnessInterventionEvent(
                HarnessInterventionKind::ActionGateBlocked,
            ))
            .expect("query_by_type");
        assert_eq!(entries.len(), 1, "expected one HarnessInterventionEvent row");
        let entry = &entries[0];
        assert_eq!(entry.agent_id, "harness_intervention");
        assert_eq!(entry.action_type.as_sql_str(), "harness_intervention_event");
    }
}
