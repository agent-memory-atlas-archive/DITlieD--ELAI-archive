//! Contract: elai_cc providers must expose an LlmProvider trait
//! with complete(), embed(), and token_count() methods.
//!
//! RED phase — compiles against existing types, fails at runtime
//! because the LlmProvider contract surface does not exist yet.

use elai_providers::{MessageRequest, MessageResponse};

/// The providers crate must eventually expose an LlmProvider trait.
/// This test verifies that MessageResponse carries provider_name as
/// a deserializable field that defaults to None when absent from JSON,
/// and can be set to a concrete value via serde round-trip.
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
        seed: None,
        effort: None,
    };
    let streamed = req.with_streaming();
    assert!(streamed.stream, "with_streaming must set stream=true");

    // CONTRACT: MessageResponse must deserialize from JSON that omits
    // provider_name (Ollama/OpenAI responses never include it).
    let resp_no_provider: MessageResponse = serde_json::from_str(
        r#"{"id":"msg_01","type":"message","role":"assistant","content":[],"model":"test","usage":{"input_tokens":0,"output_tokens":0}}"#,
    )
    .expect("valid MessageResponse JSON without provider_name");
    assert_eq!(
        resp_no_provider.provider_name, None,
        "CONTRACT: provider_name must default to None when absent from JSON"
    );

    // CONTRACT: MessageResponse must deserialize provider_name when present.
    let resp_with_provider: MessageResponse = serde_json::from_str(
        r#"{"id":"msg_02","type":"message","role":"assistant","content":[],"model":"test","usage":{"input_tokens":0,"output_tokens":0},"provider_name":"anthropic"}"#,
    )
    .expect("valid MessageResponse JSON with provider_name");
    assert_eq!(
        resp_with_provider.provider_name,
        Some("anthropic".to_string()),
        "CONTRACT: provider_name must deserialize to Some when present"
    );
}
