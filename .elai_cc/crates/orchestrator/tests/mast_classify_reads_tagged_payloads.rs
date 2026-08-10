//! E-29 MAST classifier regression: exercise externally tagged SSB payloads.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::mast::MastMode;
use elai_orchestrator::observability::mast_classify::classify_closed_run;
use elai_orchestrator::ssb::{
    FindingPayload, Ssb, SsbConfig, SsbPayload, TestOutcomePayload,
};
use elai_orchestrator::SsbViewer;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn classify_closed_run_reads_externally_tagged_rows_written_through_the_real_ssb() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let db_path = dir.path().join("e29-mast.db");
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Ssb::new(
        SsbConfig {
            db_path: db_path.clone(),
            pool_size: Some(2),
            project_id: "e29-mast".into(),
            session_id: "s-e29".into(),
        },
        clock,
    )
    .expect("ssb init");

    ssb.write_payload(
        "e29-mast",
        1,
        SsbPayload::Finding(FindingPayload {
            severity: "FAILURE".into(),
            category: "schema_violation".into(),
            message: "required field was absent".into(),
            file_path: None,
            line: None,
            failure_category: None,
            audit_step: None,
            few_shot_examples: None,
        }),
        Vec::new(),
    )
    .await
    .expect("finding row");
    ssb.write_payload(
        "e29-mast",
        2,
        SsbPayload::TestOutcome(TestOutcomePayload {
            test_name: "e29-real-ssb".into(),
            passed: true,
            duration_ms: 1,
            stdout_hash: None,
        }),
        Vec::new(),
    )
    .await
    .expect("test outcome row");

    let viewer = SsbViewer::open(&db_path).expect("viewer");
    let hits = classify_closed_run("s-e29", &viewer).expect("classify");
    assert!(!hits.is_empty(), "real SSB rows must produce classifier hits");
    assert!(hits.iter().any(|hit| {
        hit.mode == MastMode::Fm11DisobeyTaskSpec && hit.confidence_bp == 10_000
    }));
    assert!(hits
        .iter()
        .any(|hit| hit.mode == MastMode::Fm31PrematureTermination));
}
