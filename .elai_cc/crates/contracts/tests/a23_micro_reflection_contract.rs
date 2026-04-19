//! Batch 4C (A-23) — Micro-reflection emitter contract.
//!
//! Pins the narrow `ReflectionClient` trait + JSON parse path that the
//! task_runner invokes after a baby-step failure. The production emitter
//! is wired in `crates/orchestrator/src/micro_reflection_emitter.rs`; this
//! contract ensures:
//!   * `parse_reflection_json` rejects malformed / over-long JSON
//!   * `MicroReflectionPayload` round-trips via the SSB enum
//!   * a canned-JSON stub `ReflectionClient` returns a validated response
//!
//! Grounding:
//!   * crates/core/src/micro_reflection.rs MicroReflectionResponse
//!   * crates/orchestrator/src/micro_reflection_emitter.rs
//!   * crates/orchestrator/src/ssb.rs MicroReflectionPayload

use elai_core::micro_reflection::MicroReflectionResponse;
use elai_orchestrator::{
    parse_reflection_json, MicroReflectionPayload, ReflectionClient, ReflectionClientError,
    SsbActionType, SsbPayload,
};

struct CannedClient {
    json: String,
}

impl ReflectionClient for CannedClient {
    async fn reflect(
        &self,
        _prompt: &str,
    ) -> Result<MicroReflectionResponse, ReflectionClientError> {
        Ok(parse_reflection_json(&self.json)?)
    }
}

// --- (a) Valid JSON parses into a validated MicroReflectionResponse. ---

#[test]
fn contract_parse_reflection_json_accepts_valid_body() {
    let raw = r#"{"root_cause":"missing return type","faulty_line":3,"action":"add return i32"}"#;
    let parsed = parse_reflection_json(raw).expect("parse");
    assert_eq!(parsed.faulty_line, 3);
    assert_eq!(parsed.root_cause, "missing return type");
    assert_eq!(parsed.action, "add return i32");
}

// --- (b) Malformed JSON surfaces as a typed error, not a panic. ---

#[test]
fn contract_parse_reflection_json_rejects_malformed() {
    let raw = r#"{ not valid json at all }"#;
    let err = parse_reflection_json(raw);
    assert!(err.is_err(), "malformed JSON must error");
}

// --- (c) Empty root_cause is rejected by the validator. ---

#[test]
fn contract_parse_reflection_json_rejects_empty_field() {
    let raw = r#"{"root_cause":"","faulty_line":1,"action":"x"}"#;
    assert!(parse_reflection_json(raw).is_err());
}

// --- (d) MicroReflection SSB row serialises with the expected fields. ---

#[test]
fn contract_micro_reflection_payload_shape() {
    let p = MicroReflectionPayload {
        step_id: "step-1".into(),
        root_cause: "missing import".into(),
        proposed_fix: "add use crate::foo".into(),
        confidence: 0.85,
    };
    let wrapped = SsbPayload::MicroReflection(p.clone());
    assert_eq!(wrapped.action_type(), SsbActionType::MicroReflection);
    let json = serde_json::to_string(&wrapped).expect("serialize");
    assert!(json.contains("step-1"));
    assert!(json.contains("missing import"));
    let back: SsbPayload = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.action_type(), SsbActionType::MicroReflection);
}

// --- (e) A canned ReflectionClient returns a validated response. ---

#[tokio::test]
async fn contract_canned_client_returns_validated_response() {
    let raw = r#"{"root_cause":"missing return","faulty_line":2,"action":"add return"}"#;
    let client = CannedClient {
        json: raw.to_string(),
    };
    let r = client.reflect("any prompt").await.expect("reflect");
    assert_eq!(r.faulty_line, 2);
}

// --- (f) An error client surfaces a typed ReflectionClientError. ---

#[tokio::test]
async fn contract_error_client_surfaces_typed_error() {
    struct ErrClient;
    impl ReflectionClient for ErrClient {
        async fn reflect(
            &self,
            _prompt: &str,
        ) -> Result<MicroReflectionResponse, ReflectionClientError> {
            Err(ReflectionClientError::Unavailable("sidecar down".into()))
        }
    }
    let client = ErrClient;
    let r = client.reflect("prompt").await;
    assert!(matches!(
        r,
        Err(ReflectionClientError::Unavailable(_))
    ));
}
