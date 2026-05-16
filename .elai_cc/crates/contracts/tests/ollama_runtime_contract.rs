//! B-37C Phase 4M — OllamaRuntime contract.
//!
//! Pins:
//!   - canonical_name() returns "ollama".
//!   - resolve_backend("ollama") -> Ollama.
//!   - constructor returns a working RuntimeAdapter; defaults sane.
//!   - capabilities advertise streaming + tools.
//!   - health_check against an unreachable host returns Unavailable
//!     (NOT panic / NOT Ready).
//!   - source pins: POST /v1/chat/completions (primary) + /api/chat
//!     fallback + GET /api/tags health probe must all appear in source.

use elai_providers::runtime::{
    resolve_backend, OllamaRuntime, RuntimeAdapter, RuntimeBackend, RuntimeFactoryConfig,
    RuntimeHealth, RuntimePermissionMode,
};

const BACKEND: RuntimeBackend = RuntimeBackend::Ollama;

#[test]
fn canonical_name_is_ollama() {
    assert_eq!(BACKEND.canonical_name(), "ollama");
}

#[test]
fn resolve_backend_alias_returns_ollama() {
    assert_eq!(
        resolve_backend("ollama").expect("alias must resolve"),
        BACKEND
    );
}

#[test]
fn constructor_accepts_factory_config_and_reports_backend() {
    let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    let rt = OllamaRuntime::new(cfg);
    assert_eq!(rt.backend(), BACKEND);
    let caps = rt.capabilities();
    assert!(caps.supports_streaming, "ollama must advertise streaming");
    assert!(caps.supports_tools, "ollama must advertise tool support");
}

#[test]
fn default_base_url_targets_local_11434() {
    // No OLLAMA_HOST in the test env (auditor runs cargo test fresh).
    // Skip the assertion if it's set so CI on contributor laptops with
    // a custom OLLAMA_HOST doesn't false-fire.
    if std::env::var("OLLAMA_HOST").is_ok() {
        return;
    }
    let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    let rt = OllamaRuntime::new(cfg);
    assert_eq!(rt.base_url(), "http://localhost:11434");
}

#[tokio::test]
async fn health_check_unreachable_host_returns_unavailable() {
    let mut cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    cfg.base_url = Some("http://127.0.0.1:1".to_string()); // unbound port
    let rt = OllamaRuntime::new(cfg);
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
fn source_hits_expected_http_paths() {
    // Pin the wire protocol surface so a future refactor can't silently
    // change the endpoint. Source-level grep is the cheapest contract.
    let source = include_str!("../../providers/src/runtime/ollama.rs");
    assert!(
        source.contains("/v1/chat/completions"),
        "ollama must hit OpenAI-compat /v1/chat/completions"
    );
    assert!(
        source.contains("/api/chat"),
        "ollama must retain native /api/chat fallback"
    );
    assert!(
        source.contains("/api/tags"),
        "ollama health probe must hit /api/tags"
    );
    assert!(
        source.contains("OLLAMA_HOST"),
        "ollama must honor OLLAMA_HOST env override"
    );
}
