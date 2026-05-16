//! B-37B Phase 4M — CodexCliRuntime contract.
//!
//! Pins:
//!   - Default binary name is `codex`.
//!   - argv contains
//!     `exec --json --skip-git-repo-check --output-last-message <tmp> -C <cwd>`.
//!   - Final answer comes from the temp file, NOT stdout (gotchas.md
//!     L320-322 lesson).
//!   - health_check on missing binary returns Unavailable.
//!   - permission_mode AcceptEdits is the default.
//!   - Temp file path is deleted after read (MEDIUM-5 lifecycle).

use std::path::PathBuf;

use elai_providers::runtime::cli_common::default_binary_for;
use elai_providers::runtime::{
    CodexCliRuntime, RuntimeAdapter, RuntimeBackend, RuntimeFactoryConfig, RuntimeHealth,
    RuntimePermissionMode,
};

const BACKEND: RuntimeBackend = RuntimeBackend::CodexCli;

#[test]
fn default_binary_name_is_codex() {
    assert_eq!(default_binary_for(BACKEND), "codex");
}

#[test]
fn canonical_name_is_codex_cli() {
    assert_eq!(BACKEND.canonical_name(), "codex_cli");
}

#[test]
fn constructor_accepts_factory_config_and_reports_backend() {
    let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    let rt = CodexCliRuntime::new(cfg);
    assert_eq!(rt.backend(), BACKEND);
    let _caps = rt.capabilities();
}

#[tokio::test]
async fn health_check_missing_binary_returns_unavailable() {
    let mut cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    cfg.cli_path = Some(PathBuf::from("/__elai_nonexistent_codex_xyz__"));
    let rt = CodexCliRuntime::new(cfg);
    match rt.health_check().await {
        RuntimeHealth::Unavailable { .. } => {}
        other => panic!("expected Unavailable, got {other:?}"),
    }
}

#[test]
fn permission_mode_default_is_accept_edits() {
    let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    assert_eq!(cfg.permission_mode, RuntimePermissionMode::AcceptEdits);
}

#[test]
fn argv_contains_codex_exec_json_skip_git_output_last_message() {
    // Source-level grep: build_argv is private; this assertion fails
    // if any of the load-bearing flags is removed from codex_cli.rs.
    let source = include_str!("../../providers/src/runtime/codex_cli.rs");
    assert!(
        source.contains("\"exec\""),
        "codex_cli.rs must pass subcommand `exec`"
    );
    assert!(
        source.contains("\"--json\""),
        "codex_cli.rs must pass --json"
    );
    assert!(
        source.contains("\"--skip-git-repo-check\""),
        "codex_cli.rs must pass --skip-git-repo-check"
    );
    assert!(
        source.contains("\"--output-last-message\""),
        "codex_cli.rs must pass --output-last-message"
    );
    assert!(
        source.contains("\"-C\""),
        "codex_cli.rs must pass -C <cwd>"
    );
}

#[test]
fn reads_output_last_message_file_not_stdout() {
    // gotchas.md L320-322: the FINAL ANSWER lives in
    // --output-last-message <file>, not stdout. Stdout is a JSONL
    // event stream used only for liveness.
    let source = include_str!("../../providers/src/runtime/codex_cli.rs");
    assert!(
        source.contains("read_to_string(&temp_path)"),
        "codex_cli.rs MUST read the temp file (gotchas.md L320-322)"
    );
    // We also assert the file is best-effort deleted (MEDIUM-5).
    assert!(
        source.contains("remove_file(&temp_path)"),
        "codex_cli.rs MUST remove the temp file after read (MEDIUM-5)"
    );
}

#[test]
fn codex_output_temp_path_lives_in_temp_dir() {
    use elai_providers::runtime::cli_common::codex_output_temp_path;
    let p = codex_output_temp_path().expect("temp path must succeed");
    let temp = std::env::temp_dir();
    assert!(
        p.starts_with(&temp),
        "codex temp path {} must live under {}",
        p.display(),
        temp.display()
    );
    assert!(
        p.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.starts_with("elai_codex_last_message_"))
            .unwrap_or(false),
        "codex temp file name must carry the elai_codex_last_message_ prefix; got {}",
        p.display()
    );
}
