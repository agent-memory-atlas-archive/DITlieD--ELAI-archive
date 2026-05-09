// Dashboard Tier 1 CHAT window — contract tests
//
// Covers the data contract additions introduced by the chat message surface:
//   - SsbActionType::ChatMessage + ChatResponse enum variants
//   - SQL-string mapping via serde (snake_case: "chat_message", "chat_response")
//   - ChatMessagePayload + ChatResponsePayload serde round-trip
//   - SsbPayload::action_type() returns the matching variant
//   - The SSB CHECK constraint accepts the two new action_type strings when
//     a fresh database is initialised from the current CREATE_SSB_TABLE DDL
//     (guards against forgetting to update the CHECK list alongside the enum)
//
// These tests intentionally build a fresh in-memory SSB. A stale dev DB
// with an old CHECK constraint would pass silently on pre-existing variants
// but fail here the moment ChatMessage / ChatResponse are written — matching
// the counterfactual described in workflow/reasoning/phase-tier1-chat-decisions.md.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::{
    ChatMessagePayload, ChatResponsePayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
};
use tempfile::TempDir;

fn fresh_ssb(tmp: &TempDir) -> Ssb {
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "contract-project".into(),
        session_id: "contract-session".into(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    Ssb::new(cfg, clock).expect("open fresh ssb")
}

#[test]
fn chat_action_type_variants_exist_and_serialize_snake_case() {
    let msg_json = serde_json::to_string(&SsbActionType::ChatMessage).expect("serialize chat_message");
    let resp_json = serde_json::to_string(&SsbActionType::ChatResponse).expect("serialize chat_response");

    assert_eq!(msg_json, "\"chat_message\"");
    assert_eq!(resp_json, "\"chat_response\"");
}

#[test]
fn chat_action_type_deserialize_from_snake_case() {
    let msg: SsbActionType =
        serde_json::from_str("\"chat_message\"").expect("deserialize chat_message");
    let resp: SsbActionType =
        serde_json::from_str("\"chat_response\"").expect("deserialize chat_response");

    assert_eq!(msg, SsbActionType::ChatMessage);
    assert_eq!(resp, SsbActionType::ChatResponse);
}

#[test]
fn chat_message_payload_serde_roundtrip() {
    let original = ChatMessagePayload {
        session_id: "s-147708".into(),
        text: "/task rebuild the dashboard".into(),
        intent: "TASK".into(),
        project_id: "proj-alpha".into(),
        submitted_at_ms: 1_713_300_000_000,
    };
    let bytes = rmp_serde::to_vec_named(&original).expect("msgpack encode");
    let decoded: ChatMessagePayload = rmp_serde::from_slice(&bytes).expect("msgpack decode");
    assert_eq!(decoded, original);
}

#[test]
fn chat_response_payload_serde_roundtrip() {
    let original = ChatResponsePayload {
        session_id: "s-147708".into(),
        card_kind: "status".into(),
        body: "mode set to autonomous".into(),
        project_id: "proj-alpha".into(),
        emitted_at_ms: 1_713_300_000_100,
    };
    let bytes = rmp_serde::to_vec_named(&original).expect("msgpack encode");
    let decoded: ChatResponsePayload = rmp_serde::from_slice(&bytes).expect("msgpack decode");
    assert_eq!(decoded, original);
}

#[test]
fn ssb_payload_action_type_maps_chat_variants() {
    let msg = SsbPayload::ChatMessage(ChatMessagePayload {
        session_id: "s".into(),
        text: "hi".into(),
        intent: "AMBIGUOUS".into(),
        project_id: "p".into(),
        submitted_at_ms: 1,
    });
    let resp = SsbPayload::ChatResponse(ChatResponsePayload {
        session_id: "s".into(),
        card_kind: "error".into(),
        body: "boom".into(),
        project_id: "p".into(),
        emitted_at_ms: 2,
    });
    assert_eq!(msg.action_type(), SsbActionType::ChatMessage);
    assert_eq!(resp.action_type(), SsbActionType::ChatResponse);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fresh_ssb_accepts_chat_message_row() {
    // Fresh DB = CHECK constraint re-derived from the current
    // CREATE_SSB_TABLE constant. If the constant was not updated to
    // allow 'chat_message', this write will fail with SQLITE_CONSTRAINT.
    let tmp = TempDir::new().expect("tempdir");
    let ssb = fresh_ssb(&tmp);

    let payload = SsbPayload::ChatMessage(ChatMessagePayload {
        session_id: "s-147708".into(),
        text: "first chat row".into(),
        intent: "TASK".into(),
        project_id: "contract-project".into(),
        submitted_at_ms: 1_713_300_000_000,
    });

    let seq = ssb
        .write_payload("dashboard", 0, payload, vec![])
        .await
        .expect("write_payload chat_message must succeed on fresh DB");
    assert!(seq > 0, "seq must be positive; got {seq}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fresh_ssb_accepts_chat_response_row() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = fresh_ssb(&tmp);

    let payload = SsbPayload::ChatResponse(ChatResponsePayload {
        session_id: "s-147708".into(),
        card_kind: "status".into(),
        body: "routed to MODE".into(),
        project_id: "contract-project".into(),
        emitted_at_ms: 1_713_300_000_050,
    });

    let seq = ssb
        .write_payload("dashboard", 0, payload, vec![])
        .await
        .expect("write_payload chat_response must succeed on fresh DB");
    assert!(seq > 0, "seq must be positive; got {seq}");
}
