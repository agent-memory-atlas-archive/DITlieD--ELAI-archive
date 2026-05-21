//! E-51: contract test — default runtime chain must NOT include Mlx or LlamaCpp.
//!
//! Per E-51 WIRING_CONTRACT stub_ban:
//! "default runtime chain MUST NOT include Mlx or LlamaCpp without explicit opt-in"

use elai_providers::runtime::RuntimeBackend;
// s-68264 audit cleanup: orchestrator re-exports settings RuntimeConfig as RuntimeConfig
// and the runtime-chain config as RuntimeBackendConfig. Default-impl lives on the latter.
use elai_orchestrator::RuntimeBackendConfig as RuntimeConfig;

#[test]
fn default_chain_excludes_mlx_and_llamacpp() {
    let cfg = RuntimeConfig::default();
    let chain = &cfg.fallback_chain;

    assert!(
        !chain.contains(&RuntimeBackend::Mlx),
        "default fallback chain must NOT contain Mlx — it is opt-in only via \
         ELAI_RUNTIME_BACKEND=mlx or --runtime mlx flag"
    );
    assert!(
        !chain.contains(&RuntimeBackend::LlamaCpp),
        "default fallback chain must NOT contain LlamaCpp — it is opt-in only"
    );
    assert_eq!(
        chain.first(),
        Some(&RuntimeBackend::Sidecar),
        "default chain must start with Sidecar"
    );
    assert!(
        chain.contains(&RuntimeBackend::AnthropicApi),
        "default chain must still contain AnthropicApi"
    );
}

#[test]
fn default_per_role_mapping_excludes_mlx_and_llamacpp() {
    let cfg = RuntimeConfig::default();
    let roles_list = cfg.per_role.list();

    for (role, backend) in &roles_list {
        assert_ne!(
            *backend,
            RuntimeBackend::Mlx,
            "role {role:?} must not default to Mlx"
        );
        assert_ne!(
            *backend,
            RuntimeBackend::LlamaCpp,
            "role {role:?} must not default to LlamaCpp"
        );
    }
    assert_eq!(
        cfg.per_role.fallback,
        RuntimeBackend::Sidecar,
        "PerRoleRuntime fallback must be Sidecar"
    );
}
