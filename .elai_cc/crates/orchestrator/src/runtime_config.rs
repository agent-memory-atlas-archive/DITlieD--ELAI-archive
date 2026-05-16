//! Runtime backend configuration (B-37C Phase 4M.5).
//!
//! `RuntimeConfig` aggregates the per-role mapping, the global fallback
//! chain (orthogonal to `AutonomousGate.model_fallback_chain` per plan
//! lines 219 — "runtime fallback orthogonal to 4G SUBSTITUTE_MODEL"),
//! and the per-backend factory configs used to instantiate adapters on
//! demand.
//!
//! The factory_cfg map is a `BTreeMap` (deterministic ordering) but
//! `RuntimeFactoryConfig` itself does not derive PartialEq/Eq (it carries
//! `SecretString`); we therefore implement Clone manually and do NOT
//! derive PartialEq on `RuntimeConfig`. Serde is gated on a feature-free
//! shim: only `per_role` + `fallback_chain` are serializable (RuntimeFactoryConfig
//! contains secrets and PathBufs that should not be serialized in
//! settings.toml).

use std::collections::BTreeMap;

use elai_providers::runtime::{RuntimeBackend, RuntimeFactoryConfig};
use serde::{Deserialize, Serialize};

use crate::per_role_runtime::PerRoleRuntime;

/// Default ordered chain consulted when a primary backend is unavailable.
///
/// `Sidecar` first (locally hosted, fast, low marginal cost), then
/// `AnthropicApi` (cloud fallback), then `ClaudeCode` (CLI). The chain
/// is deliberately short — long chains amplify latency. Operators can
/// override per-role chains in B-37D once the `/runtime fallback` CLI
/// command lands; until then the same chain is used for every role.
#[must_use]
pub fn default_fallback_chain() -> Vec<RuntimeBackend> {
    vec![
        RuntimeBackend::Sidecar,
        RuntimeBackend::AnthropicApi,
        RuntimeBackend::ClaudeCode,
    ]
}

/// Top-level runtime configuration. Drives `RuntimeOrchestrator`.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Per-role primary backend mapping.
    pub per_role: PerRoleRuntime,
    /// Ordered fallback chain consulted when the primary backend fails
    /// or is unavailable. Orthogonal to
    /// `AutonomousGateConfig.model_fallback_chain` (model substitution
    /// within a working backend) — runtime fallback is one level up
    /// (whole-backend swap).
    pub fallback_chain: Vec<RuntimeBackend>,
    /// Factory configs keyed by backend. `RuntimeOrchestrator` clones the
    /// entry for the target backend on first use, instantiates the
    /// adapter via `create_runtime`, and caches it.
    pub factory_cfg: BTreeMap<RuntimeBackend, RuntimeFactoryConfig>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        // Pre-populate factory_cfg with default configs for every known
        // backend. `RuntimeFactoryConfig::for_backend` zeroes optional
        // fields, so cloud / CLI adapters will surface
        // `RuntimeAdapterError::MissingField` on first `complete()` call
        // until the operator configures real values via the (future)
        // `/runtime set` command. This is intentional — `health_check`
        // returns `Unavailable`, not a panic, when fields are missing.
        let mut factory_cfg = BTreeMap::new();
        for backend in RuntimeBackend::all() {
            factory_cfg.insert(*backend, RuntimeFactoryConfig::for_backend(*backend));
        }
        Self {
            per_role: PerRoleRuntime::default(),
            fallback_chain: default_fallback_chain(),
            factory_cfg,
        }
    }
}

impl RuntimeConfig {
    /// Replace the factory config for one backend.
    pub fn set_factory_config(
        &mut self,
        backend: RuntimeBackend,
        cfg: RuntimeFactoryConfig,
    ) {
        self.factory_cfg.insert(backend, cfg);
    }

    /// Returns a clone of the factory config for `backend`, creating a
    /// default-populated entry if absent.
    #[must_use]
    pub fn factory_config_for(&self, backend: RuntimeBackend) -> RuntimeFactoryConfig {
        self.factory_cfg
            .get(&backend)
            .cloned()
            .unwrap_or_else(|| RuntimeFactoryConfig::for_backend(backend))
    }
}

/// Persistable view of `RuntimeConfig` — strips out factory configs
/// (which may carry secrets) and serializes only the runtime-policy
/// surface (per-role mapping + fallback chain). This is the shape
/// settings.toml should round-trip through.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeConfigPolicyView {
    pub per_role: PerRoleRuntime,
    pub fallback_chain: Vec<RuntimeBackend>,
}

impl Default for RuntimeConfigPolicyView {
    fn default() -> Self {
        Self {
            per_role: PerRoleRuntime::default(),
            fallback_chain: default_fallback_chain(),
        }
    }
}

impl From<&RuntimeConfig> for RuntimeConfigPolicyView {
    fn from(cfg: &RuntimeConfig) -> Self {
        Self {
            per_role: cfg.per_role.clone(),
            fallback_chain: cfg.fallback_chain.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_chain_starts_with_sidecar() {
        let chain = default_fallback_chain();
        assert_eq!(chain.first(), Some(&RuntimeBackend::Sidecar));
        assert!(chain.contains(&RuntimeBackend::AnthropicApi));
    }

    #[test]
    fn default_runtime_config_has_factory_for_every_backend() {
        let cfg = RuntimeConfig::default();
        for backend in RuntimeBackend::all() {
            assert!(
                cfg.factory_cfg.contains_key(backend),
                "factory_cfg missing {backend:?}"
            );
        }
    }

    #[test]
    fn policy_view_round_trips_through_json() {
        let view = RuntimeConfigPolicyView::default();
        let json = serde_json::to_string(&view).expect("serialize");
        let decoded: RuntimeConfigPolicyView =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(view, decoded);
    }

    #[test]
    fn set_factory_config_overrides_existing() {
        let mut cfg = RuntimeConfig::default();
        let mut new_factory = RuntimeFactoryConfig::for_backend(RuntimeBackend::Sidecar);
        new_factory.model = Some("override-model".into());
        cfg.set_factory_config(RuntimeBackend::Sidecar, new_factory);
        assert_eq!(
            cfg.factory_config_for(RuntimeBackend::Sidecar).model,
            Some("override-model".into())
        );
    }
}
