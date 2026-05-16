//! B-37C Phase 4M — VllmRuntime contract.
//!
//! Pins:
//!   - canonical_name() returns "vllm".
//!   - resolve_backend("vllm") -> Vllm.
//!   - constructor + capabilities (streaming + tools).
//!   - health_check against an unreachable host returns Unavailable.
//!   - source pins: POST /v1/chat/completions + GET /health +
//!     GET /v1/models all appear in source.

use elai_providers::runtime::{
    resolve_backend, RuntimeAdapter, RuntimeBackend, RuntimeFactoryConfig, RuntimeHealth,
    RuntimePermissionMode, VllmRuntime,
};

const BACKEND: RuntimeBackend = RuntimeBackend::Vllm;

#[test]
fn canonical_name_is_vllm() {
    assert_eq!(BACKEND.canonical_name(), "vllm");
}

#[test]
fn resolve_backend_alias_returns_vllm() {
    assert_eq!(
        resolve_backend("vllm").expect("alias must resolve"),
        BACKEND
    );
}

#[test]
fn constructor_accepts_factory_config_and_reports_backend() {
    let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    let rt = VllmRuntime::new(cfg);
    assert_eq!(rt.backend(), BACKEND);
    let caps = rt.capabilities();
    assert!(caps.supports_streaming, "vllm must advertise streaming");
    assert!(caps.supports_tools, "vllm must advertise tool support");
    assert!(
        caps.json_schema_mode,
        "vllm supports OpenAI-style JSON schema mode"
    );
}

#[test]
fn default_base_url_targets_local_8000() {
    let cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    let rt = VllmRuntime::new(cfg);
    assert_eq!(rt.base_url(), "http://localhost:8000");
}

#[tokio::test]
async fn health_check_unreachable_host_returns_unavailable() {
    let mut cfg = RuntimeFactoryConfig::for_backend(BACKEND);
    cfg.base_url = Some("http://127.0.0.1:1".to_string()); // unbound port
    let rt = VllmRuntime::new(cfg);
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
    let source = include_str!("../../providers/src/runtime/vllm.rs");
    assert!(
        source.contains("/v1/chat/completions"),
        "vllm must hit OpenAI-compat /v1/chat/completions"
    );
    assert!(
        source.contains("/health"),
        "vllm must probe /health endpoint"
    );
    assert!(
        source.contains("/v1/models"),
        "vllm must fall back to /v1/models for health"
    );
}
