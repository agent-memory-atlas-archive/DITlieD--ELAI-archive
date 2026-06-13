//! E-26 contract: SglangRuntime adapter surface.
//!
//! Verifies the SGLang HTTP runtime adapter wired by E-26:
//! 1. `SglangRuntime` constructs without panicking under
//!    `RuntimeFactoryConfig::for_backend(RuntimeBackend::Sglang)`.
//! 2. `backend()` returns `RuntimeBackend::Sglang`.
//! 3. `canonical_name()` returns `"sglang"`.
//! 4. Default base URL is `http://localhost:30000` (no env override).
//! 5. `ELAI_SGLANG_BASE_URL` env var overrides the default.
//! 6. Capabilities advertise json_schema_mode + streaming + tools.
//! 7. `StructuredOutputBackend` snake_case wire tags match
//!    docs.sglang.ai (`xgrammar` / `outlines` / `lm-format-enforcer`).
//! 8. `resolve_backend("sglang")` round-trips to `RuntimeBackend::Sglang`.
//! 9. `resolve_backend("sgl")` alias works.
//! 10. `RuntimeBackend::all()` includes `Sglang`.
//! 11. `create_runtime(Sglang)` returns a live adapter (not
//!     NotImplemented/MissingField).
//!
//! Health-check + complete + stream against a live SGLang server are
//! RUNPOD-PENDING — these tests cover the host-side adapter shape only.

use std::sync::Mutex;

use elai_providers::runtime::{
    create_runtime, resolve_backend, RuntimeAdapter, RuntimeBackend, RuntimeFactoryConfig,
    SglangRuntime, SglangRuntimeConfig, StructuredOutputBackend,
};

/// Process-wide lock for env-mutating tests. Prevents racing on
/// ELAI_SGLANG_BASE_URL across parallel test threads.
static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn sglang_runtime_constructs_without_panic() {
    let _g = ENV_LOCK.lock().expect("env lock");
    let prev = std::env::var("ELAI_SGLANG_BASE_URL").ok();
    std::env::remove_var("ELAI_SGLANG_BASE_URL");

    let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::Sglang);
    let rt = SglangRuntime::new(cfg);
    assert_eq!(rt.backend(), RuntimeBackend::Sglang);

    if let Some(v) = prev {
        std::env::set_var("ELAI_SGLANG_BASE_URL", v);
    }
}

#[test]
fn sglang_canonical_name_is_sglang() {
    assert_eq!(SglangRuntime::canonical_name(), "sglang");
    assert_eq!(RuntimeBackend::Sglang.canonical_name(), "sglang");
}

#[test]
fn sglang_default_base_url_is_30000() {
    let _g = ENV_LOCK.lock().expect("env lock");
    let prev = std::env::var("ELAI_SGLANG_BASE_URL").ok();
    std::env::remove_var("ELAI_SGLANG_BASE_URL");

    let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::Sglang);
    let rt = SglangRuntime::new(cfg);
    assert_eq!(rt.base_url(), "http://localhost:30000");

    if let Some(v) = prev {
        std::env::set_var("ELAI_SGLANG_BASE_URL", v);
    }
}

#[test]
fn sglang_env_overrides_default_base_url() {
    let _g = ENV_LOCK.lock().expect("env lock");
    let prev = std::env::var("ELAI_SGLANG_BASE_URL").ok();
    std::env::set_var("ELAI_SGLANG_BASE_URL", "http://override.test:31000");

    let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::Sglang);
    let rt = SglangRuntime::new(cfg);
    assert_eq!(rt.base_url(), "http://override.test:31000");

    std::env::remove_var("ELAI_SGLANG_BASE_URL");
    if let Some(v) = prev {
        std::env::set_var("ELAI_SGLANG_BASE_URL", v);
    }
}

#[test]
fn sglang_capabilities_advertise_json_schema_streaming_tools() {
    let _g = ENV_LOCK.lock().expect("env lock");
    let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::Sglang);
    let rt = SglangRuntime::new(cfg);
    let caps = rt.capabilities();
    assert!(caps.json_schema_mode, "SGLang advertises json_schema_mode");
    assert!(caps.supports_streaming, "SGLang advertises streaming");
    assert!(caps.supports_tools, "SGLang advertises tools");
}

#[test]
fn structured_output_backend_wire_tags_match_docs() {
    // docs.sglang.ai/backend/structured_outputs accepts these snake_case
    // tags on extra_body.backend.
    assert_eq!(StructuredOutputBackend::XGrammar.as_str(), "xgrammar");
    assert_eq!(StructuredOutputBackend::Outlines.as_str(), "outlines");
    assert_eq!(StructuredOutputBackend::Lmfe.as_str(), "lm-format-enforcer");
    // Default is XGrammar per blog.mlc.ai 2026-05-04 (XGrammar is the
    // default in SGLang, vLLM, TRT-LLM, MLC-LLM).
    assert_eq!(
        StructuredOutputBackend::default(),
        StructuredOutputBackend::XGrammar
    );
}

#[test]
fn sglang_runtime_config_default_uses_canonical_url_and_xgrammar() {
    let cfg = SglangRuntimeConfig::default();
    assert_eq!(cfg.base_url, "http://localhost:30000");
    assert_eq!(
        cfg.structured_output_backend,
        StructuredOutputBackend::XGrammar
    );
    assert!(cfg.api_key.is_none());
}

#[test]
fn resolve_backend_sglang_canonical_and_aliases() {
    assert_eq!(
        resolve_backend("sglang").expect("canonical sglang"),
        RuntimeBackend::Sglang
    );
    assert_eq!(
        resolve_backend("sgl").expect("alias sgl"),
        RuntimeBackend::Sglang
    );
    assert_eq!(
        resolve_backend("sglang_runtime").expect("alias sglang_runtime"),
        RuntimeBackend::Sglang
    );
    // Case insensitive (mirrors B-37 precedent).
    assert_eq!(
        resolve_backend("SGLANG").expect("uppercase"),
        RuntimeBackend::Sglang
    );
}

#[test]
fn runtime_backend_all_includes_sglang() {
    let all = RuntimeBackend::all();
    assert!(
        all.iter().any(|b| *b == RuntimeBackend::Sglang),
        "RuntimeBackend::all() must include Sglang"
    );
    // E-26 bumped the count to 17; E-39 (commit 929148e) added 3 more
    // cloud-floor backends: DeepSeekV4Flash, CerebrasFree, GroqFree.
    // All three are in RuntimeBackend::all() (types.rs:212-214) with
    // create_runtime returning NotImplemented until RUNPOD-PENDING resolves.
    assert_eq!(all.len(), 20);
}

#[test]
fn create_runtime_sglang_returns_live_adapter() {
    let _g = ENV_LOCK.lock().expect("env lock");
    let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::Sglang);
    let adapter = create_runtime(cfg).expect("Sglang should construct via factory");
    assert_eq!(adapter.backend(), RuntimeBackend::Sglang);
    let caps = adapter.capabilities();
    assert!(caps.json_schema_mode);
}
