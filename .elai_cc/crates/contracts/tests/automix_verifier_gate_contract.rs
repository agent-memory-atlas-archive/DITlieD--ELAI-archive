//! E-39 contract: automix verifier gate returns a 3-arm verdict and
//! the SSB row carries `CostEventKind::AutomixVerifierVerdict`.
//!
//! Drives `verify_before_escalate` with a deterministic stub client,
//! writes the verdict to a temp Ssb, reads back and asserts the kind
//! discriminator and verdict string in details_json.
//!
//! Production entrypoint: elai run -> drive_task cloud-dispatch branch
//! -> `verify_before_escalate` call site (cloud_mode.rs / task_runner.rs
//! escalation hook, E-39 wired path).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::cost::automix_gate::{
    verify_before_escalate, AutomixError, AutomixVerdict, VerifierClient,
};
use elai_orchestrator::ssb::{
    CostEventKind, CostTokenEventPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

struct StubClient(&'static str);

impl VerifierClient for StubClient {
    fn verify(&self, _system: &str, _user: &str) -> Result<String, AutomixError> {
        Ok(self.0.to_string())
    }
}

// WIRE-DARK[E-39]: verify_before_escalate has no production caller in
// cloud_mode.rs L3-failed escalation hook yet (OPTION B deferred disjoint
// slice). Resolves when the call is added to the escalation hook. See
// E-39-decisions.md.
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_automix_verifier_gate_escalate_ssb_roundtrip() {
    // 1. Drive the real verifier gate with a stub that returns "escalate".
    let client = StubClient(r#"{"verdict":"escalate","reason":"output is too short"}"#);
    let (verdict, reason) =
        verify_before_escalate(&client, "implement a sorting algorithm", "x = 1")
            .expect("verify_before_escalate must succeed with stub client");
    assert_eq!(verdict, AutomixVerdict::Escalate);
    assert!(!reason.is_empty(), "reason must be non-empty");

    // 2. Write the off-agent artifact SSB row.
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "e39-automix".into(),
        session_id: "e39-automix-session".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let payload = SsbPayload::CostTokenEvent(CostTokenEventPayload {
        kind: CostEventKind::AutomixVerifierVerdict,
        session_id: Some("e39-automix-session".into()),
        task_id: Some("task-automix-001".into()),
        details_json: serde_json::json!({
            "verdict": verdict.as_str(),
            "reason": reason,
            "retry_count": 0
        }),
        ts_ms: 1_717_000_002_000,
    });
    ssb.write_payload("cost", 0, payload, Vec::new())
        .await
        .expect("write");

    // 3. Read back and assert.
    let rows = ssb
        .query_by_type(SsbActionType::CostTokenEvent)
        .expect("query");
    assert_eq!(rows.len(), 1, "expected exactly one CostTokenEvent row");
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::CostTokenEvent(p) => {
            assert_eq!(
                p.kind,
                CostEventKind::AutomixVerifierVerdict,
                "kind discriminator must be AutomixVerifierVerdict"
            );
            assert_eq!(p.details_json["verdict"], serde_json::json!("escalate"));
            let reason_str = p.details_json["reason"]
                .as_str()
                .expect("reason must be a string");
            assert!(!reason_str.is_empty(), "reason must be non-empty in SSB");
        }
        other => panic!("expected CostTokenEvent, got {other:?}"),
    }
}
