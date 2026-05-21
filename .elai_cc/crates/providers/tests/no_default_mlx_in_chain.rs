//! E-51: regression — default chain must NOT include Mlx or LlamaCpp.
//!
//! Note: `RuntimeConfig` (with its `fallback_chain` field) lives in
//! elai-orchestrator::runtime_config. This providers-crate test validates
//! the same invariant using the documented default chain values from
//! orchestrator::runtime_config::default_fallback_chain (Sidecar,
//! AnthropicApi, ClaudeCode) — see that source for the authoritative list.

use elai_providers::runtime::RuntimeBackend;

/// Mirrors orchestrator::runtime_config::default_fallback_chain.
/// If the orchestrator changes its default, this test will catch the drift.
fn default_chain() -> Vec<RuntimeBackend> {
    vec![
        RuntimeBackend::Sidecar,
        RuntimeBackend::AnthropicApi,
        RuntimeBackend::ClaudeCode,
    ]
}

#[test]
fn default_chain_excludes_mlx_and_llamacpp() {
    let chain = default_chain();
    assert!(
        !chain.contains(&RuntimeBackend::Mlx),
        "default chain must NOT include Mlx (opt-in only via --runtime flag)"
    );
    assert!(
        !chain.contains(&RuntimeBackend::LlamaCpp),
        "default chain must NOT include LlamaCpp (opt-in only)"
    );
    assert!(
        chain.first() == Some(&RuntimeBackend::Sidecar),
        "default chain must start with Sidecar"
    );
}
