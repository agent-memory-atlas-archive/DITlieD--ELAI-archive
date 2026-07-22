// J-39 T2 contract: NodeAdmissionEvent SSB emit + T1 admission_ceiling_threshold intact.
// Exercises the SAME production fn wired at task_runner.rs SPLIT_TASK phase_sizing branch.
// No reimplemented emit logic in the test body.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::admission_ceiling_threshold;
use elai_orchestrator::ssb::{
    NodeAdmissionEventKind, NodeAdmissionEventPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
    SsbReader,
};
use elai_orchestrator::emit_node_admission_event;

fn os_clock() -> Arc<dyn ClockProvider> {
    Arc::new(OsClock)
}

async fn build_ssb(test_id: &str) -> (Arc<Ssb>, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("create tempdir for SSB");
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: format!("proj-{}", test_id),
        session_id: format!("sess-{}", test_id),
    };
    let ssb = Ssb::new(cfg, os_clock()).expect("Ssb::new");
    (Arc::new(ssb), dir)
}

fn decode_node_admission_rows(
    ssb: &Arc<Ssb>,
    kind: NodeAdmissionEventKind,
) -> Vec<NodeAdmissionEventPayload> {
    let rows = ssb
        .query_by_type(SsbActionType::NodeAdmissionEvent(kind))
        .expect("query_by_type");
    rows.iter()
        .filter_map(|r| {
            let decoded: Result<SsbPayload, _> = rmp_serde::from_slice(&r.payload);
            decoded.ok()
        })
        .filter_map(|p| match p {
            SsbPayload::NodeAdmissionEvent(p) => Some(p),
            _ => None,
        })
        .filter(|p| p.kind == kind)
        .collect()
}

#[tokio::test]
async fn contract_admitted_emits_node_admission_event() {
    let (ssb, _dir) = build_ssb("admitted").await;

    emit_node_admission_event(
        &ssb,
        NodeAdmissionEventKind::Admitted,
        "j39-contract-admitted",
        1000,
        6000,
        serde_json::json!({
            "should_split": false,
            "sub_count": 0,
        }),
    );

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let rows = decode_node_admission_rows(&ssb, NodeAdmissionEventKind::Admitted);
    assert!(
        !rows.is_empty(),
        "CONTRACT: emit_node_admission_event(Admitted) must produce at least one NodeAdmissionEvent SSB row"
    );
    let row = &rows[0];
    assert_eq!(row.kind, NodeAdmissionEventKind::Admitted);
    assert_eq!(row.step_id, "j39-contract-admitted");
    assert_eq!(row.token_estimate, 1000);
    assert_eq!(row.ceiling_threshold, 6000);
}

#[tokio::test]
async fn contract_split_forced_emits_node_admission_event() {
    let (ssb, _dir) = build_ssb("split-forced").await;

    emit_node_admission_event(
        &ssb,
        NodeAdmissionEventKind::SplitForced,
        "j39-contract-split",
        8000,
        6000,
        serde_json::json!({
            "should_split": true,
            "sub_count": 3,
        }),
    );

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let rows = decode_node_admission_rows(&ssb, NodeAdmissionEventKind::SplitForced);
    assert!(
        !rows.is_empty(),
        "CONTRACT: emit_node_admission_event(SplitForced) must produce at least one NodeAdmissionEvent SSB row"
    );
    let row = &rows[0];
    assert_eq!(row.kind, NodeAdmissionEventKind::SplitForced);
    assert_eq!(row.step_id, "j39-contract-split");
    assert_eq!(row.token_estimate, 8000);
    assert_eq!(row.ceiling_threshold, 6000);
}

#[test]
fn contract_t1_admission_ceiling_threshold_still_reachable() {
    let ratio = 0.6_f32;
    let threshold = admission_ceiling_threshold(ratio);
    assert!(
        threshold > 0,
        "T1 intact: admission_ceiling_threshold must return a positive value"
    );
}
