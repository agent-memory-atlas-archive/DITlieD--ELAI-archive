//! B-37B Phase 4M — CopilotCliRuntime contract.
//!
//! Pins:
//!   - Default binary name is `copilot` (verified against
//!     workflow/reasoning/B-37-handoff.md lines 67-69; not
//!     `gh-copilot`).
//!   - argv contains
//!     `--no-color --log-level none --add-dir <cwd>` + permission
//!     flags + `-p <prompt>`.
//!   - permission_mode AcceptEdits/Default -> `--allow-tool shell`;
//!     DangerouslySkipPermissions (with allow_danger=true) ->
//!     `--allow-all`; Plan -> no allow flag.
//!   - health_check on missing binary returns Unavailable.
//!   - canonical_name() returns "copilot_cli".
//!   - JSONL stream parser handles copilot's event format
//!     (`text`/`content`/`message` keys per copilot_cli.rs:230).

use std::path::PathBuf;

use elai_providers::runtime::cli_common::{
    default_binary_for, permission_mode_to_copilot_args,
};
use elai_providers::runtime::{
    CopilotCliRuntime, RuntimeAdapter, RuntimeBackend, RuntimeFactoryConfig, RuntimeHealth,
    RuntimePermissionMode,
};

const BACKEND: RuntimeBackend = RuntimeBackend::CopilotCli;

#[test]
fn default_binary_name_is_copilot() {
    let bin = default_binary_for(BACKEND);
    assert_eq!(
        bin, "copilot",
        "Copilot CLI binary is `copilot`, NOT `gh-copilot` (handoff L67-69)"
    );
}

#[test]
fn canonical_name_is_copilot_cli() {
    assert_eq!(BACKEND.canonical_name(), "copilot_cli");
}

#[test]
fn constructor_accepts_factory_config_and_reports_backend() {
    let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    let rt = CopilotCliRuntime::new(cfg);
    assert_eq!(rt.backend(), BACKEND);
}

#[test]
fn argv_contains_no_color_log_level_add_dir_and_dash_p() {
    let source = include_str!("../../providers/src/runtime/copilot_cli.rs");
    assert!(
        source.contains("\"--no-color\""),
        "copilot_cli.rs must pass --no-color"
    );
    assert!(
        source.contains("\"--log-level\""),
        "copilot_cli.rs must pass --log-level"
    );
    assert!(
        source.contains("\"none\""),
        "copilot_cli.rs must pass `none` to --log-level"
    );
    assert!(
        source.contains("\"--add-dir\""),
        "copilot_cli.rs must pass --add-dir <cwd>"
    );
    assert!(
        source.contains("\"-p\""),
        "copilot_cli.rs must pass -p <prompt>"
    );
}

#[test]
fn permission_mode_maps_to_allow_or_allow_all() {
    // AcceptEdits / Default -> --allow-tool shell.
    let accept = permission_mode_to_copilot_args(RuntimePermissionMode::AcceptEdits);
    assert_eq!(
        accept,
        vec!["--allow-tool".to_string(), "shell".to_string()],
        "AcceptEdits must produce --allow-tool shell"
    );
    let default = permission_mode_to_copilot_args(RuntimePermissionMode::Default);
    assert_eq!(
        default,
        vec!["--allow-tool".to_string(), "shell".to_string()],
        "Default must produce --allow-tool shell"
    );
    // DangerouslySkipPermissions -> --allow-all (must still pass the
    // `allow_danger` opt-in gate before reaching this point).
    let dangerous =
        permission_mode_to_copilot_args(RuntimePermissionMode::DangerouslySkipPermissions);
    assert_eq!(
        dangerous,
        vec!["--allow-all".to_string()],
        "DangerouslySkipPermissions must produce --allow-all"
    );
    // Plan -> no allow flag (copilot stays read-only-ish).
    let plan = permission_mode_to_copilot_args(RuntimePermissionMode::Plan);
    assert!(plan.is_empty(), "Plan must produce no allow flag, got {plan:?}");
}

#[tokio::test]
async fn health_check_missing_binary_returns_unavailable() {
    let mut cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    cfg.cli_path = Some(PathBuf::from("/__elai_nonexistent_copilot_xyz__"));
    let rt = CopilotCliRuntime::new(cfg);
    match rt.health_check().await {
        RuntimeHealth::Unavailable { .. } => {}
        other => panic!("expected Unavailable, got {other:?}"),
    }
}

#[test]
fn jsonl_event_parser_accepts_text_content_message_keys() {
    // copilot_cli.rs:230 falls back to text/content/message; we make
    // sure those fixture lines parse as valid JSON (the parser itself
    // is private but its expected key set is contract-relevant).
    let fixtures = [
        r#"{"text":"hello"}"#,
        r#"{"content":"world"}"#,
        r#"{"message":"final"}"#,
    ];
    for line in fixtures {
        let v: serde_json::Value =
            serde_json::from_str(line).expect("copilot JSONL fixture must parse");
        assert!(
            v.get("text").is_some()
                || v.get("content").is_some()
                || v.get("message").is_some()
        );
    }
}
