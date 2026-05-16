//! B-37B Phase 4M — KiroCliRuntime contract.
//!
//! Pins:
//!   - Default binary name is `kiro-cli` (WITH hyphen — paper L223,
//!     L298-299; the canonical backend name `kiro_cli` is separate).
//!   - argv contains `chat --no-interactive`.
//!   - Alias map maps both `kiro` and `kiro_cli` to RuntimeBackend::KiroCli.
//!   - health_check on missing binary returns Unavailable.
//!   - canonical_name() returns "kiro_cli".
//!   - permission_mode maps to a `--trust-tools=...` or
//!     `--trust-all-tools` flag (paper L225).

use std::path::PathBuf;

use elai_providers::runtime::cli_common::default_binary_for;
use elai_providers::runtime::{
    resolve_backend, KiroCliRuntime, RuntimeAdapter, RuntimeBackend, RuntimeFactoryConfig,
    RuntimeHealth, RuntimePermissionMode,
};

const BACKEND: RuntimeBackend = RuntimeBackend::KiroCli;

#[test]
fn default_binary_name_is_kiro_cli_with_hyphen() {
    let bin = default_binary_for(BACKEND);
    assert_eq!(
        bin, "kiro-cli",
        "Kiro binary name has a HYPHEN (paper L223+L298-299), got `{bin}`"
    );
}

#[test]
fn canonical_name_is_kiro_cli() {
    assert_eq!(BACKEND.canonical_name(), "kiro_cli");
}

#[test]
fn alias_map_resolves_both_kiro_and_kiro_cli() {
    assert_eq!(
        resolve_backend("kiro").expect("alias `kiro`"),
        RuntimeBackend::KiroCli,
        "alias `kiro` must resolve to KiroCli"
    );
    assert_eq!(
        resolve_backend("kiro_cli").expect("alias `kiro_cli`"),
        RuntimeBackend::KiroCli,
        "alias `kiro_cli` must resolve to KiroCli"
    );
}

#[test]
fn constructor_accepts_factory_config_and_reports_backend() {
    let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    let rt = KiroCliRuntime::new(cfg);
    assert_eq!(rt.backend(), BACKEND);
}

#[test]
fn argv_contains_chat_no_interactive() {
    let source = include_str!("../../providers/src/runtime/kiro_cli.rs");
    assert!(source.contains("\"chat\""), "kiro_cli.rs must pass subcommand `chat`");
    assert!(
        source.contains("\"--no-interactive\""),
        "kiro_cli.rs must pass --no-interactive (paper L225)"
    );
}

#[test]
fn permission_mode_drives_trust_flag() {
    // Source-level check: the `trust_flag` function picks the right
    // CLI argument for each permission mode.
    let source = include_str!("../../providers/src/runtime/kiro_cli.rs");
    assert!(
        source.contains("--trust-tools=fs,bash") || source.contains("\"--trust-tools="),
        "kiro_cli.rs must emit --trust-tools=... for AcceptEdits"
    );
    assert!(
        source.contains("--trust-all-tools"),
        "kiro_cli.rs must emit --trust-all-tools for DangerouslySkipPermissions"
    );
    // Plan / Default mode emits NO trust flag — the trust_flag fn
    // returns None for those branches; verify via source pattern.
    assert!(
        source.contains("RuntimePermissionMode::Plan | RuntimePermissionMode::Default => None"),
        "kiro_cli.rs Plan/Default branch must return None"
    );
    // Sentinel: ensure the enum variants we depend on still exist.
    let _ = RuntimePermissionMode::AcceptEdits;
    let _ = RuntimePermissionMode::DangerouslySkipPermissions;
}

#[tokio::test]
async fn health_check_missing_binary_returns_unavailable() {
    let mut cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    cfg.cli_path = Some(PathBuf::from("/__elai_nonexistent_kirocli_xyz__"));
    let rt = KiroCliRuntime::new(cfg);
    match rt.health_check().await {
        RuntimeHealth::Unavailable { .. } => {}
        other => panic!("expected Unavailable, got {other:?}"),
    }
}
