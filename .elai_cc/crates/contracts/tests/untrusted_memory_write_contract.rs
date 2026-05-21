// E-30: Untrusted TextSource memory-write guard contract.
//
// Pins the contract that `LabeledText` and `TextSource::is_trusted_for_memory_write()`
// correctly classify all source variants so downstream gates can refuse writes
// from untrusted sources without inspecting content.
//
// Attack pattern: an external tool (MCP, shell, file read) returns adversarial
// content that tricks the agent into writing it to memory, enabling indirect
// prompt injection across sessions.
//
// Policy (E-30):
//   - Trusted for memory write: UserPrompt, OperatorApproved.
//   - NOT trusted: ExternalTool, UntrustedFile, AssistantGenerated.
//
// The distinction is conservative by design: AssistantGenerated is not trusted
// because it may itself have been influenced by untrusted tool output.

use elai_orchestrator::{LabeledText, TextSource};

// ---------------------------------------------------------------------------
// Trust classification contract
// ---------------------------------------------------------------------------

#[test]
fn user_prompt_is_trusted_for_memory_write() {
    assert!(
        TextSource::UserPrompt.is_trusted_for_memory_write(),
        "UserPrompt must be trusted for memory writes"
    );
}

#[test]
fn operator_approved_is_trusted_for_memory_write() {
    assert!(
        TextSource::OperatorApproved.is_trusted_for_memory_write(),
        "OperatorApproved must be trusted for memory writes"
    );
}

#[test]
fn external_tool_is_not_trusted_for_memory_write() {
    let source = TextSource::ExternalTool { tool_name: "bash".into() };
    assert!(
        !source.is_trusted_for_memory_write(),
        "ExternalTool must NOT be trusted for memory writes"
    );
}

#[test]
fn untrusted_file_is_not_trusted_for_memory_write() {
    let source = TextSource::UntrustedFile { path: "/tmp/evil.txt".into() };
    assert!(
        !source.is_trusted_for_memory_write(),
        "UntrustedFile must NOT be trusted for memory writes"
    );
}

#[test]
fn assistant_generated_is_not_trusted_for_memory_write() {
    // Conservative: assistant output may itself have been influenced by
    // untrusted tool output. Only operator-confirmed sources are trusted.
    assert!(
        !TextSource::AssistantGenerated.is_trusted_for_memory_write(),
        "AssistantGenerated must NOT be trusted for memory writes (indirect injection risk)"
    );
}

// ---------------------------------------------------------------------------
// LabeledText construction contract
// ---------------------------------------------------------------------------

#[test]
fn labeled_text_from_user_has_user_prompt_source() {
    let lt = LabeledText::from_user("hello");
    assert_eq!(lt.source, TextSource::UserPrompt);
    assert_eq!(lt.content, "hello");
}

#[test]
fn labeled_text_from_assistant_has_assistant_source() {
    let lt = LabeledText::from_assistant("response");
    assert_eq!(lt.source, TextSource::AssistantGenerated);
}

#[test]
fn labeled_text_from_tool_has_external_tool_source() {
    let lt = LabeledText::from_tool("bash", "output");
    match &lt.source {
        TextSource::ExternalTool { tool_name } => {
            assert_eq!(tool_name, "bash");
        }
        other => panic!("expected ExternalTool, got {other:?}"),
    }
    assert_eq!(lt.content, "output");
}

#[test]
fn labeled_text_serde_round_trip() {
    let original = LabeledText::from_tool("mcp-server", "some tool output");
    let json = serde_json::to_string(&original).expect("serialize");
    let decoded: LabeledText = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, decoded);
}

#[test]
fn all_source_variants_serde_round_trip() {
    let sources = vec![
        TextSource::UserPrompt,
        TextSource::AssistantGenerated,
        TextSource::ExternalTool { tool_name: "test-tool".into() },
        TextSource::UntrustedFile { path: "/tmp/test".into() },
        TextSource::OperatorApproved,
    ];
    for source in sources {
        let json = serde_json::to_string(&source).expect("serialize");
        let decoded: TextSource = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(source, decoded);
    }
}
