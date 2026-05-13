//! Contract: `MessageRequest::with_seed` plumbing.
//!
//! Asserts the three layers of the determinism seed pipeline:
//! 1. The builder writes `Option<u64>` from a `u32` argument (sidecar cap).
//! 2. Serde forwards `seed` on the Anthropic `/v1/messages` JSON body via
//!    `AnthropicRequestProfile::render_json_body`.
//! 3. The OpenAI-compat `chat.completions` request builder also forwards
//!    `seed` (previously dropped).

use elai_providers::{
    build_chat_completion_request_for_test, AnthropicRequestProfile, MessageRequest,
};
use serde_json::json;

fn minimal_request() -> MessageRequest {
    let raw = json!({
        "model": "test-model",
        "max_tokens": 16,
        "messages": [
            {
                "role": "user",
                "content": [{"type": "text", "text": "hi"}]
            }
        ]
    });
    serde_json::from_value(raw).expect("minimal MessageRequest fixture must deserialize")
}

#[test]
fn with_seed_sets_field() {
    let request = minimal_request().with_seed(42);
    assert_eq!(request.seed, Some(42_u64));
}

#[test]
fn with_seed_serialises_into_anthropic_body() {
    let request = minimal_request().with_seed(42);
    let body = AnthropicRequestProfile::default()
        .render_json_body(&request)
        .expect("anthropic body must render for a well-formed request");
    let seed = body
        .get("seed")
        .expect("seed key must be present on the anthropic wire body");
    assert_eq!(seed.as_u64(), Some(42_u64));
}

#[test]
fn with_seed_serialises_into_openai_compat_body() {
    let request = minimal_request().with_seed(42);
    let body = build_chat_completion_request_for_test(&request);
    let seed = body
        .get("seed")
        .expect("seed key must be present on the openai-compat wire body");
    assert_eq!(seed.as_u64(), Some(42_u64));
}
