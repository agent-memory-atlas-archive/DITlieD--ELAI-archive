//! B-37B Phase 4M — ClaudeCodeRuntime contract.
//!
//! Pins:
//!   - Default binary name is `claude` (NO hyphen — paper L298 says
//!     kiro is the only hyphenated binary).
//!   - argv contains `--output-format stream-json --verbose
//!     --include-partial-messages` (verified WebSearch in
//!     workflow/reasoning/B-37-handoff.md L64-67).
//!   - health_check on a non-existent binary returns Unavailable, never
//!     panics.
//!   - permission_mode maps to the right `--permission-mode <arg>`
//!     value (`acceptEdits` / `bypassPermissions` / `default`).
//!   - canonical_name() returns "claude_code".
//!   - ClaudeCodeRuntime::new(cfg) compiles and produces a usable
//!     trait object.

use std::path::PathBuf;

use elai_providers::runtime::cli_common::{
    default_binary_for, permission_mode_to_claude_arg,
};
use elai_providers::runtime::{
    ClaudeCodeRuntime, RuntimeAdapter, RuntimeBackend, RuntimeFactoryConfig, RuntimeHealth,
    RuntimePermissionMode,
};

const BACKEND: RuntimeBackend = RuntimeBackend::ClaudeCode;

#[test]
fn default_binary_name_is_claude_no_hyphen() {
    assert_eq!(
        default_binary_for(BACKEND),
        "claude",
        "ClaudeCode binary is `claude`, NOT `claude-code` (paper L298)"
    );
}

#[test]
fn canonical_name_is_claude_code() {
    assert_eq!(BACKEND.canonical_name(), "claude_code");
}

#[test]
fn constructor_accepts_factory_config_and_reports_backend() {
    let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    let rt = ClaudeCodeRuntime::new(cfg);
    assert_eq!(rt.backend(), BACKEND);
    // capabilities() is cheap, MUST not panic.
    let caps = rt.capabilities();
    assert!(caps.supports_tools, "ClaudeCode advertises supports_tools");
    assert!(caps.supports_streaming, "ClaudeCode advertises streaming");
}

#[tokio::test]
async fn health_check_when_binary_missing_returns_unavailable() {
    let mut cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    // Sentinel path that cannot exist; resolve_binary() permits an
    // explicit path even if missing (cli_common.rs:95-98) but
    // health_check_cli_binary then inspects `is_file()` and reports
    // Unavailable.
    cfg.cli_path = Some(PathBuf::from("/__elai_nonexistent_claude_xyz__"));
    let rt = ClaudeCodeRuntime::new(cfg);
    match rt.health_check().await {
        RuntimeHealth::Unavailable { .. } => {}
        other => panic!("expected Unavailable for missing binary, got {other:?}"),
    }
}

#[test]
fn permission_mode_maps_to_claude_flag() {
    assert_eq!(
        permission_mode_to_claude_arg(RuntimePermissionMode::AcceptEdits),
        "acceptEdits"
    );
    assert_eq!(
        permission_mode_to_claude_arg(RuntimePermissionMode::DangerouslySkipPermissions),
        "bypassPermissions"
    );
    assert_eq!(
        permission_mode_to_claude_arg(RuntimePermissionMode::Default),
        "default"
    );
    assert_eq!(
        permission_mode_to_claude_arg(RuntimePermissionMode::Plan),
        "default"
    );
}

#[tokio::test]
async fn complete_with_dangerous_mode_refuses_without_allow_danger() {
    // Backend-specific assertion: the central permission gate fires
    // before any subprocess is spawned. We invoke complete() with a
    // sentinel cli_path so we don't accidentally execute a real
    // `claude` binary on the test box. The gate MUST refuse with
    // PermissionDenied before we get to spawn.
    use elai_providers::runtime::{CompletionConfig, Message, MessageRole, RuntimeAdapterError};
    let mut cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    cfg.cli_path = Some(PathBuf::from("/__elai_nonexistent_claude_xyz__"));
    cfg.permission_mode = RuntimePermissionMode::DangerouslySkipPermissions;
    cfg.allow_danger = false;
    let rt = ClaudeCodeRuntime::new(cfg);
    let res = rt
        .complete(
            vec![Message {
                role: MessageRole::User,
                content: "hi".to_string(),
            }],
            CompletionConfig::default(),
        )
        .await;
    match res {
        Err(RuntimeAdapterError::PermissionDenied { .. }) => {}
        other => panic!(
            "expected PermissionDenied with allow_danger=false, got {other:?}"
        ),
    }
}

#[test]
fn argv_contains_required_claude_flags() {
    // claude_code.rs:68 `build_argv` is private; we re-verify the
    // contract by reading the source via string match. This is a
    // light-touch grep-style assertion that catches regressions if
    // someone deletes one of the required flags.
    let source = include_str!(
        "../../providers/src/runtime/claude_code.rs"
    );
    assert!(
        source.contains("\"--output-format\""),
        "claude_code.rs must pass --output-format"
    );
    assert!(
        source.contains("\"stream-json\""),
        "claude_code.rs must use stream-json output"
    );
    assert!(
        source.contains("\"--verbose\""),
        "claude_code.rs must pass --verbose"
    );
    assert!(
        source.contains("\"--include-partial-messages\""),
        "claude_code.rs must pass --include-partial-messages"
    );
    assert!(
        source.contains("\"--permission-mode\""),
        "claude_code.rs must pass --permission-mode"
    );
}
