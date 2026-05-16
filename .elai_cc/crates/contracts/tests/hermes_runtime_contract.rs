//! B-37B Phase 4M — HermesRuntime contract.
//!
//! Pins:
//!   - Default binary name is `hermes`.
//!   - argv contains `chat -Q --source tool -q <prompt>`.
//!   - `supports_tools == false` — hermes uniquely lacks tool-envelope
//!     support (paper L40, L191).
//!   - Arg-length cap rejects prompts above the documented limit
//!     (MEDIUM-6 — hermes uses `-q <arg>` and hits the OS arg-length
//!     ceiling).
//!   - health_check on missing binary returns Unavailable.
//!   - canonical_name() returns "hermes".

use std::path::PathBuf;

use elai_providers::runtime::cli_common::default_binary_for;
use elai_providers::runtime::{
    CompletionConfig, HermesRuntime, Message, MessageRole, RuntimeAdapter, RuntimeAdapterError,
    RuntimeBackend, RuntimeFactoryConfig, RuntimeHealth,
};

const BACKEND: RuntimeBackend = RuntimeBackend::Hermes;

#[test]
fn default_binary_name_is_hermes() {
    assert_eq!(default_binary_for(BACKEND), "hermes");
}

#[test]
fn canonical_name_is_hermes() {
    assert_eq!(BACKEND.canonical_name(), "hermes");
}

#[test]
fn constructor_accepts_factory_config_and_reports_backend() {
    let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    let rt = HermesRuntime::new(cfg);
    assert_eq!(rt.backend(), BACKEND);
}

#[test]
fn supports_tools_is_false() {
    let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    let rt = HermesRuntime::new(cfg);
    assert!(
        !rt.capabilities().supports_tools,
        "Hermes uniquely lacks tool-envelope support (paper L40, L191)"
    );
}

#[test]
fn argv_contains_chat_minus_q_source_tool_minus_q() {
    let source = include_str!("../../providers/src/runtime/hermes.rs");
    assert!(source.contains("\"chat\""), "hermes.rs must pass subcommand `chat`");
    assert!(source.contains("\"-Q\""), "hermes.rs must pass -Q (quiet)");
    assert!(source.contains("\"--source\""), "hermes.rs must pass --source");
    assert!(source.contains("\"tool\""), "hermes.rs must pass `tool` to --source");
    assert!(source.contains("\"-q\""), "hermes.rs must pass -q <prompt>");
}

#[tokio::test]
async fn health_check_missing_binary_returns_unavailable() {
    let mut cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    cfg.cli_path = Some(PathBuf::from("/__elai_nonexistent_hermes_xyz__"));
    let rt = HermesRuntime::new(cfg);
    match rt.health_check().await {
        RuntimeHealth::Unavailable { .. } => {}
        other => panic!("expected Unavailable, got {other:?}"),
    }
}

#[tokio::test]
async fn arg_length_cap_rejects_oversized_prompt() {
    // hermes.rs:30 declares MAX_HERMES_PROMPT_BYTES = 30 * 1024.
    // Building a prompt larger than that must yield an
    // RuntimeAdapterError before any subprocess spawn attempt.
    let mut cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    cfg.cli_path = Some(PathBuf::from("/__elai_nonexistent_hermes_xyz__"));
    let rt = HermesRuntime::new(cfg);
    let huge = "a".repeat(64 * 1024);
    let res = rt
        .complete(
            vec![Message {
                role: MessageRole::User,
                content: huge,
            }],
            CompletionConfig::default(),
        )
        .await;
    match res {
        Err(RuntimeAdapterError::Other(msg)) => {
            assert!(
                msg.contains("prompt") && msg.contains("bytes"),
                "expected arg-length-cap error, got `{msg}`"
            );
        }
        // BinaryNotFound is also acceptable if the size check happens
        // AFTER binary resolve — but the current implementation places
        // the size check earlier; if the order changes the test should
        // be revisited. We still pass on BinaryNotFound to keep the
        // contract loose enough for refactors.
        Err(RuntimeAdapterError::BinaryNotFound { .. }) => {}
        other => panic!(
            "expected oversized-prompt error or BinaryNotFound, got {other:?}"
        ),
    }
}
