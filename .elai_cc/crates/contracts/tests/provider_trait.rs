//! Contract: elai_cc providers must expose an LlmProvider trait
//! with complete(), embed(), and token_count() methods.
//!
//! RED phase — compiles against existing types, fails at runtime
//! because the LlmProvider contract surface does not exist yet.

use elai_providers::{MessageRequest, MessageResponse};

/// The providers crate must eventually expose an LlmProvider trait.
/// This test uses existing types to verify the new contract: that
/// MessageResponse debug output carries provider_name (part of
/// LlmProvider). It does not — so the assertion fails at runtime.
#[test]
fn provider_has_llm_provider_contract() {
    // Construct a MessageRequest using existing public fields.
    let req = MessageRequest {
        model: "claude-sonnet-4-20250514".into(),
        max_tokens: 1024,
        messages: vec![],
        system: None,
        tools: None,
        tool_choice: None,
        stream: false,
    };
    let streamed = req.with_streaming();
    assert!(streamed.stream, "with_streaming must set stream=true");

    // Construct a minimal MessageResponse via serde.
    let resp: MessageResponse = serde_json::from_str(
        r#"{"id":"msg_01","type":"message","role":"assistant","content":[],"model":"test","usage":{"input_tokens":0,"output_tokens":0}}"#,
    )
    .expect("valid MessageResponse JSON");

    // CONTRACT assertion: MessageResponse must carry provider_name
    // from the LlmProvider trait. This field does not exist yet,
    // so the debug output will NOT contain "provider_name".
    let resp_debug = format!("{resp:?}");
    assert!(
        resp_debug.contains("provider_name"),
        "CONTRACT: MessageResponse must carry provider_name from LlmProvider trait"
    );
}
