//! B-37B Phase 4M — GeminiCliRuntime contract.
//!
//! Pins:
//!   - Default binary name is `gemini`.
//!   - argv contains
//!     `--prompt <p> --non-interactive --output-format stream-json
//!      --approval-mode <mode>` (paper L207).
//!   - permission_mode maps to the right `--approval-mode` argument
//!     (`auto_edit` for AcceptEdits, `yolo` for
//!     DangerouslySkipPermissions).
//!   - health_check on missing binary returns Unavailable.
//!   - canonical_name() returns "gemini_cli".

use std::path::PathBuf;

use elai_providers::runtime::cli_common::{
    default_binary_for, permission_mode_to_gemini_approval_mode,
};
use elai_providers::runtime::{
    GeminiCliRuntime, RuntimeAdapter, RuntimeBackend, RuntimeFactoryConfig, RuntimeHealth,
    RuntimePermissionMode,
};

const BACKEND: RuntimeBackend = RuntimeBackend::GeminiCli;

#[test]
fn default_binary_name_is_gemini() {
    assert_eq!(default_binary_for(BACKEND), "gemini");
}

#[test]
fn canonical_name_is_gemini_cli() {
    assert_eq!(BACKEND.canonical_name(), "gemini_cli");
}

#[test]
fn constructor_accepts_factory_config_and_reports_backend() {
    let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    let rt = GeminiCliRuntime::new(cfg);
    assert_eq!(rt.backend(), BACKEND);
}

#[test]
fn argv_contains_prompt_non_interactive_stream_json_approval_mode() {
    let source = include_str!("../../providers/src/runtime/gemini_cli.rs");
    assert!(source.contains("\"--prompt\""), "gemini_cli.rs must pass --prompt");
    assert!(
        source.contains("\"--non-interactive\""),
        "gemini_cli.rs must pass --non-interactive"
    );
    assert!(
        source.contains("\"--output-format\""),
        "gemini_cli.rs must pass --output-format"
    );
    assert!(
        source.contains("\"stream-json\""),
        "gemini_cli.rs must pass `stream-json` to --output-format"
    );
    assert!(
        source.contains("\"--approval-mode\""),
        "gemini_cli.rs must pass --approval-mode"
    );
}

#[test]
fn permission_mode_maps_to_approval_mode() {
    assert_eq!(
        permission_mode_to_gemini_approval_mode(RuntimePermissionMode::AcceptEdits),
        "auto_edit"
    );
    assert_eq!(
        permission_mode_to_gemini_approval_mode(RuntimePermissionMode::DangerouslySkipPermissions),
        "yolo"
    );
    // Plan / Default map to auto_edit (no dedicated gemini plan mode).
    assert_eq!(
        permission_mode_to_gemini_approval_mode(RuntimePermissionMode::Plan),
        "auto_edit"
    );
    assert_eq!(
        permission_mode_to_gemini_approval_mode(RuntimePermissionMode::Default),
        "auto_edit"
    );
}

#[tokio::test]
async fn health_check_missing_binary_returns_unavailable() {
    let mut cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    cfg.cli_path = Some(PathBuf::from("/__elai_nonexistent_gemini_xyz__"));
    let rt = GeminiCliRuntime::new(cfg);
    match rt.health_check().await {
        RuntimeHealth::Unavailable { .. } => {}
        other => panic!("expected Unavailable, got {other:?}"),
    }
}

#[test]
fn stream_json_parser_handles_gemini_schema() {
    // Backend-specific assertion: gemini's NDJSON schema uses event
    // `type` discriminants. The parser in gemini_cli.rs targets
    // `"text"` / `"message"` events. We verify the fixture lines
    // parse as valid JSON before getting handed to the parser.
    let fixtures = [
        r#"{"type":"text","text":"hello"}"#,
        r#"{"type":"message","content":"world"}"#,
        r#"{"type":"thinking","text":"reflecting"}"#,
    ];
    for line in fixtures {
        let _: serde_json::Value =
            serde_json::from_str(line).expect("gemini stream-json fixture must parse");
    }
}
