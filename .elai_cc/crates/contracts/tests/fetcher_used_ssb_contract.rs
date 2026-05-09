// Track A.4 contract test: WebResearchStepPayload.fetcher field.
//
// GREEN: fetcher field is wired end-to-end through the orchestrator.
//   - `fetcher: "reqwest"` or `fetcher: "obscura"` on each WebResearchStep row.
//   - Old rows without the field deserialize to "" via #[serde(default)].
//
// Test coverage:
//   A.4.a (struct): construct WebResearchStepPayload with fetcher="reqwest",
//     verify msgpack round-trip preserves the field.
//   A.4.b (struct): same with fetcher="obscura".
//   A.4.c (backward compat): deserialize JSON that lacks the fetcher field,
//     assert it defaults to "".
//   A.4.d (end-to-end): drive SsbWebResearchStepEmitter with reqwest event,
//     read back the SSB row, assert fetcher="reqwest".
//   A.4.e (end-to-end, obscura): same with fetcher="obscura".
//     Gated on #[cfg(feature = "obscura")] so default-feature CI stays green.

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader, WebResearchStepPayload,
};
use elai_orchestrator::web_research_orchestrator::{
    WebResearchStepEmitter, WebResearchStepEvent,
};
use elai_orchestrator::SsbWebResearchStepEmitter;

// ---------------------------------------------------------------------------
// Shared helper: open an in-memory SSB in a tempdir.
// ---------------------------------------------------------------------------

fn new_ssb(db_path: std::path::PathBuf) -> Arc<Ssb> {
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "contracts".into(),
        session_id: "test-a4".into(),
    };
    Arc::new(Ssb::new(cfg, clock).expect("ssb"))
}

// ---------------------------------------------------------------------------
// A.4.a — WebResearchStepPayload with fetcher="reqwest" round-trips.
// ---------------------------------------------------------------------------

#[test]
fn web_research_step_payload_fetcher_reqwest_roundtrips() {
    let payload = WebResearchStepPayload {
        agent_id: "agent-1".into(),
        node_id: 7,
        action: "fetch".into(),
        phi: 0.85,
        psi: 0.72,
        iteration: 1,
        sources_fetched: 3,
        sources_used: 2,
        fetcher: "reqwest".into(),
        stage: String::new(),
        stage_phase: String::new(),
        stage_payload: None,
    };

    let wrapped = SsbPayload::WebResearchStep(payload);
    assert_eq!(
        wrapped.action_type(),
        SsbActionType::WebResearchStep,
        "action_type must be WebResearchStep",
    );

    let bytes = rmp_serde::to_vec_named(&wrapped)
        .expect("WebResearchStep msgpack serialize");
    let decoded: SsbPayload =
        rmp_serde::from_slice(&bytes).expect("WebResearchStep msgpack deserialize");

    match decoded {
        SsbPayload::WebResearchStep(p) => {
            assert_eq!(p.fetcher, "reqwest", "fetcher must survive msgpack round-trip");
            assert_eq!(p.agent_id, "agent-1", "agent_id must survive round-trip");
            assert_eq!(p.sources_fetched, 3, "sources_fetched must survive round-trip");
        }
        other => panic!("expected WebResearchStep; got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// A.4.b — WebResearchStepPayload with fetcher="obscura" round-trips.
// ---------------------------------------------------------------------------

#[test]
fn web_research_step_payload_fetcher_obscura_roundtrips() {
    let payload = WebResearchStepPayload {
        agent_id: "agent-2".into(),
        node_id: 9,
        action: "fetch".into(),
        phi: 0.90,
        psi: 0.60,
        iteration: 2,
        sources_fetched: 1,
        sources_used: 1,
        fetcher: "obscura".into(),
        stage: String::new(),
        stage_phase: String::new(),
        stage_payload: None,
    };

    let bytes = rmp_serde::to_vec_named(&SsbPayload::WebResearchStep(payload))
        .expect("obscura serialize");
    let decoded: SsbPayload =
        rmp_serde::from_slice(&bytes).expect("obscura deserialize");

    match decoded {
        SsbPayload::WebResearchStep(p) => {
            assert_eq!(p.fetcher, "obscura", "fetcher='obscura' must survive round-trip");
        }
        other => panic!("expected WebResearchStep; got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// A.4.c — backward compat: rows without fetcher field (pre-patch JSON)
//          deserialize to fetcher="" via #[serde(default)].
// ---------------------------------------------------------------------------

#[test]
fn web_research_step_payload_missing_fetcher_defaults_to_empty() {
    // Pre-patch row — no fetcher field present.
    let json = r#"{
        "agent_id": "agent-old",
        "node_id": 1,
        "action": "fetch",
        "phi": 0.5,
        "psi": 0.5,
        "iteration": 1,
        "sources_fetched": 2,
        "sources_used": 1
    }"#;

    let p: WebResearchStepPayload =
        serde_json::from_str(json).expect("old row without fetcher must deserialize via serde(default)");

    assert_eq!(
        p.fetcher, "",
        "missing fetcher field must default to empty string via #[serde(default)]",
    );
}

// ---------------------------------------------------------------------------
// A.4.d — end-to-end: SsbWebResearchStepEmitter writes fetcher="reqwest" to
//          the SSB; querying the row back yields the correct fetcher label.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn emitter_writes_reqwest_fetcher_to_ssb() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let ssb = new_ssb(tmp.path().join("ssb.db"));
    let emitter = SsbWebResearchStepEmitter::new(ssb.clone());

    emitter
        .emit(WebResearchStepEvent {
            node_id: 42,
            action: "iterate".into(),
            phi: 0.7,
            psi: 0.6,
            iteration: 1,
            sources_fetched: 2,
            sources_used: 1,
            fetcher: "reqwest".into(),
        })
        .await;

    let rows = ssb
        .query_by_type(SsbActionType::WebResearchStep)
        .expect("query WebResearchStep rows");
    assert_eq!(rows.len(), 1, "expected exactly one WebResearchStep row");

    let decoded: SsbPayload =
        rmp_serde::from_slice(rows[0].payload.as_ref()).expect("decode row payload");
    match decoded {
        SsbPayload::WebResearchStep(p) => {
            assert_eq!(p.fetcher, "reqwest", "SSB row fetcher must be 'reqwest'");
            assert_eq!(p.node_id, 42, "SSB row node_id must match");
        }
        other => panic!("expected WebResearchStep; got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// A.4.e — end-to-end (obscura): same assertion for fetcher="obscura".
//          Gated on feature = "obscura" so default-feature CI stays green.
// ---------------------------------------------------------------------------

#[cfg(feature = "obscura")]
#[tokio::test]
async fn emitter_writes_obscura_fetcher_to_ssb() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let ssb = new_ssb(tmp.path().join("ssb.db"));
    let emitter = SsbWebResearchStepEmitter::new(ssb.clone());

    emitter
        .emit(WebResearchStepEvent {
            node_id: 99,
            action: "iterate".into(),
            phi: 0.8,
            psi: 0.7,
            iteration: 1,
            sources_fetched: 1,
            sources_used: 1,
            fetcher: "obscura".into(),
        })
        .await;

    let rows = ssb
        .query_by_type(SsbActionType::WebResearchStep)
        .expect("query WebResearchStep rows");
    assert_eq!(rows.len(), 1, "expected exactly one WebResearchStep row");

    let decoded: SsbPayload =
        rmp_serde::from_slice(rows[0].payload.as_ref()).expect("decode row payload");
    match decoded {
        SsbPayload::WebResearchStep(p) => {
            assert_eq!(p.fetcher, "obscura", "SSB row fetcher must be 'obscura'");
            assert_eq!(p.node_id, 99, "SSB row node_id must match");
        }
        other => panic!("expected WebResearchStep; got {other:?}"),
    }
}
