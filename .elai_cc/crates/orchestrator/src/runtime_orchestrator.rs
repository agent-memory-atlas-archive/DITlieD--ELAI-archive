//! Runtime orchestrator (B-37C Phase 4M.5).
//!
//! `RuntimeOrchestrator` owns:
//!   - the `RuntimeConfig` (per-role mapping + fallback chain + factory configs),
//!   - a lazy cache of instantiated `RuntimeAdapter` instances (one per
//!     backend),
//!   - per-backend health snapshots used to short-circuit fallback walks.
//!
//! It is consumed by `task_runner` / `agent_role_dispatcher` at the
//! per-spawn boundary: given an `AgentRole`, return an `Arc<dyn
//! RuntimeAdapter>` ready for `complete`/`stream`. Production wiring of
//! the orchestrator into `task_runner.rs` is deferred to B-37D; B-37C
//! ships only the library surface plus inline tests.
//!
//! Concurrency model:
//!   - `adapters` is `tokio::sync::RwLock` (async-held) because adapter
//!     instantiation may perform blocking I/O (read auth cache, build
//!     reqwest::Client). Lookups grab a read lock first; the write lock
//!     is taken only on cache miss.
//!   - `health` is `tokio::sync::RwLock` for the same reason —
//!     `health_check` is an async fn on the adapter.
//!
//! No `std::sync::Mutex` is held across `.await`.

use std::collections::BTreeMap;
use std::sync::Arc;

use elai_ipc_types::AgentRole;
use elai_providers::runtime::{
    create_runtime, runtime_event_sink, CompletionConfig, CompletionResponse, Message,
    RuntimeAdapter, RuntimeAdapterError, RuntimeBackend, RuntimeEvent, RuntimeHealth,
};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::runtime_config::RuntimeConfig;

/// Errors surfaced by the runtime orchestrator. CLI subprocess errors are
/// flattened from `RuntimeAdapterError` so callers can program against a
/// single error type per the trait shape.
#[derive(Debug, Error)]
pub enum RuntimeOrchestratorError {
    /// Every backend in the fallback chain failed `complete`/`stream` or
    /// reported `Unavailable`. The `tried` list preserves order so
    /// dashboards can render the full attempt trail.
    #[error("all backends failed for role; tried {tried:?}")]
    AllBackendsFailed { tried: Vec<RuntimeBackend> },
    /// Adapter factory returned an error during lazy instantiation. The
    /// orchestrator records this and walks the fallback chain rather
    /// than bubbling it up directly, but the variant is surfaced when
    /// every chain attempt produces a factory error.
    #[error("adapter factory error for {backend:?}: {source}")]
    FactoryError {
        backend: RuntimeBackend,
        #[source]
        source: RuntimeAdapterError,
    },
    /// The runtime config is malformed (e.g. empty fallback chain when
    /// no primary mapping exists).
    #[error("invalid runtime config: {msg}")]
    InvalidConfig { msg: String },
}

/// Lifecycle event emitted by the orchestrator for downstream SSB
/// writers. Surfaced as `Vec<RuntimeOrchestratorEvent>` from the
/// `try_complete` return + `health_snapshot` poll so the wiring layer
/// (B-37D) can map them to the `runtime_fallback`,
/// `runtime_health_changed`, `cli_subprocess_spawned`, and
/// `cli_subprocess_killed` SSB rows without coupling this crate to the
/// SSB writer API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeOrchestratorEvent {
    /// One step of the fallback chain attempted.
    FallbackStep {
        role: AgentRole,
        primary: RuntimeBackend,
        attempted: Vec<RuntimeBackend>,
        succeeded: Option<RuntimeBackend>,
        reason: String,
    },
    /// Per-backend health transition.
    HealthChanged {
        backend: RuntimeBackend,
        prev: RuntimeHealth,
        next: RuntimeHealth,
    },
    /// CLI adapter spawned a subprocess.
    CliSubprocessSpawned {
        backend: RuntimeBackend,
        pid: u32,
        argv_redacted: Vec<String>,
    },
    /// CLI adapter killed a subprocess.
    CliSubprocessKilled {
        backend: RuntimeBackend,
        pid: u32,
        exit_code: Option<i32>,
        reason: String,
    },
}

/// Snapshot of a `try_complete` outcome.
#[derive(Debug, Clone)]
pub struct CompleteOutcome {
    pub response: CompletionResponse,
    pub used_backend: RuntimeBackend,
    pub events: Vec<RuntimeOrchestratorEvent>,
}

/// Runtime orchestrator. Cheap to clone — only the `Arc<RwLock<...>>`
/// caches are bumped, not the config.
pub struct RuntimeOrchestrator {
    config: RwLock<RuntimeConfig>,
    /// Lazy adapter cache keyed by backend.
    adapters: RwLock<BTreeMap<RuntimeBackend, Arc<dyn RuntimeAdapter>>>,
    /// Most recent observed health per backend; used to short-circuit
    /// chain walks and detect health transitions.
    health: RwLock<BTreeMap<RuntimeBackend, RuntimeHealth>>,
}

impl RuntimeOrchestrator {
    /// Build a fresh orchestrator from the given config. Does not
    /// instantiate any adapter — they are created lazily on first use.
    #[must_use]
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config: RwLock::new(config),
            adapters: RwLock::new(BTreeMap::new()),
            health: RwLock::new(BTreeMap::new()),
        }
    }

    /// Replace the underlying config (used by `/runtime set` in B-37D).
    /// Adapter cache is cleared so callers see the new factory configs
    /// on the next `for_role` lookup.
    pub async fn set_config(&self, config: RuntimeConfig) {
        let mut cfg_guard = self.config.write().await;
        *cfg_guard = config;
        let mut adapters_guard = self.adapters.write().await;
        adapters_guard.clear();
    }

    /// Borrow a clone of the current runtime config.
    pub async fn snapshot_config(&self) -> RuntimeConfig {
        self.config.read().await.clone()
    }

    /// Return the adapter for `role`'s primary backend, lazily creating
    /// it if missing. On factory failure walks the fallback chain in
    /// order, returning `AllBackendsFailed` if every entry errors out.
    pub async fn for_role(
        &self,
        role: AgentRole,
    ) -> Result<Arc<dyn RuntimeAdapter>, RuntimeOrchestratorError> {
        let cfg = self.config.read().await;
        let primary = cfg.per_role.get(role);
        let mut tried: Vec<RuntimeBackend> = Vec::new();
        // Walk chain: primary first, then each fallback entry not already
        // attempted.
        let chain: Vec<RuntimeBackend> = std::iter::once(primary)
            .chain(cfg.fallback_chain.iter().copied().filter(|b| *b != primary))
            .collect();
        drop(cfg);

        let mut last_err: Option<RuntimeAdapterError> = None;
        for backend in chain {
            tried.push(backend);
            match self.ensure_adapter(backend).await {
                Ok(adapter) => return Ok(adapter),
                Err(err) => {
                    warn!(
                        target = "runtime_orchestrator",
                        ?role,
                        ?backend,
                        error = %err,
                        "adapter factory failed; trying next backend in chain"
                    );
                    last_err = Some(err);
                    continue;
                }
            }
        }

        if let Some(source) = last_err {
            // Surface the most recent factory error in the chain (callers
            // can still see `tried` via `AllBackendsFailed` if they
            // re-walk).
            Err(RuntimeOrchestratorError::FactoryError {
                backend: *tried.last().expect("chain is non-empty"),
                source,
            })
        } else {
            Err(RuntimeOrchestratorError::AllBackendsFailed { tried })
        }
    }

    /// Drive a `complete` call for `role` with full fallback. On each
    /// attempt that fails with `Unavailable` / `NotImplemented` /
    /// `HealthTimeout` / `Network`, the chain advances. Returns the
    /// response plus the lifecycle event trail for the SSB writer.
    pub async fn try_complete(
        &self,
        role: AgentRole,
        messages: Vec<Message>,
        config: CompletionConfig,
    ) -> Result<CompleteOutcome, RuntimeOrchestratorError> {
        let cfg = self.config.read().await;
        let primary = cfg.per_role.get(role);
        let chain: Vec<RuntimeBackend> = std::iter::once(primary)
            .chain(cfg.fallback_chain.iter().copied().filter(|b| *b != primary))
            .collect();
        drop(cfg);

        let mut events: Vec<RuntimeOrchestratorEvent> = Vec::new();
        let mut tried: Vec<RuntimeBackend> = Vec::new();
        let mut last_reason = String::new();

        for backend in chain {
            tried.push(backend);
            let adapter = match self.ensure_adapter(backend).await {
                Ok(a) => a,
                Err(err) => {
                    last_reason = format!("factory: {err}");
                    debug!(
                        target = "runtime_orchestrator",
                        ?backend,
                        error = %err,
                        "factory error; continuing fallback walk"
                    );
                    continue;
                }
            };

            match adapter.complete(messages.clone(), config.clone()).await {
                Ok(response) => {
                    let reason = if backend == primary {
                        "primary backend succeeded".to_string()
                    } else {
                        format!(
                            "fallback succeeded after primary {primary:?} failed: {last_reason}"
                        )
                    };
                    events.push(RuntimeOrchestratorEvent::FallbackStep {
                        role,
                        primary,
                        attempted: tried.clone(),
                        succeeded: Some(backend),
                        reason: reason.clone(),
                    });
                    // B-37 WARN-4 fix — emit to the global SSB sink so
                    // the dashboard's runtime_fallback panel + auditor
                    // wiring-contract observable_effect predicate fire.
                    runtime_event_sink().record(RuntimeEvent::Fallback {
                        role: agent_role_to_snake(role),
                        primary,
                        attempted: tried.clone(),
                        succeeded: Some(backend),
                        reason,
                    });
                    info!(
                        target = "runtime_orchestrator",
                        ?role,
                        ?backend,
                        primary = ?primary,
                        "complete succeeded"
                    );
                    return Ok(CompleteOutcome {
                        response,
                        used_backend: backend,
                        events,
                    });
                }
                Err(err) => {
                    let recoverable = is_recoverable(&err);
                    last_reason = format!("complete: {err}");
                    warn!(
                        target = "runtime_orchestrator",
                        ?backend,
                        ?role,
                        error = %err,
                        recoverable,
                        "complete failed"
                    );
                    if !recoverable {
                        // Non-recoverable: stop walking, surface error
                        // wrapped as AllBackendsFailed for uniformity.
                        let reason =
                            format!("non-recoverable error on {backend:?}: {err}");
                        events.push(RuntimeOrchestratorEvent::FallbackStep {
                            role,
                            primary,
                            attempted: tried.clone(),
                            succeeded: None,
                            reason: reason.clone(),
                        });
                        runtime_event_sink().record(RuntimeEvent::Fallback {
                            role: agent_role_to_snake(role),
                            primary,
                            attempted: tried.clone(),
                            succeeded: None,
                            reason,
                        });
                        return Err(RuntimeOrchestratorError::AllBackendsFailed { tried });
                    }
                    continue;
                }
            }
        }

        let exhausted_reason = format!("exhausted fallback chain: {last_reason}");
        events.push(RuntimeOrchestratorEvent::FallbackStep {
            role,
            primary,
            attempted: tried.clone(),
            succeeded: None,
            reason: exhausted_reason.clone(),
        });
        runtime_event_sink().record(RuntimeEvent::Fallback {
            role: agent_role_to_snake(role),
            primary,
            attempted: tried.clone(),
            succeeded: None,
            reason: exhausted_reason,
        });
        Err(RuntimeOrchestratorError::AllBackendsFailed { tried })
    }

    /// Probe `health_check` on every cached adapter (plus the primary
    /// backend of every role even if not yet cached). Returns a vector
    /// of `(backend, RuntimeHealth)` plus any `HealthChanged` events for
    /// transitions vs the previous snapshot.
    pub async fn health_snapshot(
        &self,
    ) -> Vec<(RuntimeBackend, RuntimeHealth)> {
        // Determine the set of backends to probe: every backend with a
        // known factory config (which `RuntimeConfig::default` populates
        // for all 11) so dashboards see complete coverage.
        let backends: Vec<RuntimeBackend> = {
            let cfg = self.config.read().await;
            cfg.factory_cfg.keys().copied().collect()
        };

        let mut snapshot: Vec<(RuntimeBackend, RuntimeHealth)> = Vec::with_capacity(backends.len());
        let mut transitions: Vec<RuntimeOrchestratorEvent> = Vec::new();

        for backend in backends {
            let next = match self.ensure_adapter(backend).await {
                Ok(adapter) => adapter.health_check().await,
                Err(err) => RuntimeHealth::Unavailable {
                    reason: format!("factory error: {err}"),
                },
            };

            let prev_opt = {
                let prev_guard = self.health.read().await;
                prev_guard.get(&backend).cloned()
            };
            if let Some(prev) = prev_opt.as_ref() {
                if prev != &next {
                    transitions.push(RuntimeOrchestratorEvent::HealthChanged {
                        backend,
                        prev: prev.clone(),
                        next: next.clone(),
                    });
                    // B-37 WARN-4 fix — emit to the SSB sink for
                    // dashboard surfacing.
                    runtime_event_sink().record(RuntimeEvent::HealthChanged {
                        backend,
                        prev: prev.clone(),
                        next: next.clone(),
                    });
                }
            } else {
                // First observation — emit a transition from a synthetic
                // "Unavailable: never observed" sentinel so dashboards
                // pick up the initial state.
                let synthetic_prev = RuntimeHealth::Unavailable {
                    reason: "first observation".into(),
                };
                transitions.push(RuntimeOrchestratorEvent::HealthChanged {
                    backend,
                    prev: synthetic_prev.clone(),
                    next: next.clone(),
                });
                runtime_event_sink().record(RuntimeEvent::HealthChanged {
                    backend,
                    prev: synthetic_prev,
                    next: next.clone(),
                });
            }

            {
                let mut health_guard = self.health.write().await;
                health_guard.insert(backend, next.clone());
            }
            snapshot.push((backend, next));
        }

        // The caller in B-37D will pair `snapshot` with `transitions`
        // via a parallel `take_pending_events` helper; for now we just
        // log them at debug level so test code can validate behaviour
        // without an SSB writer.
        if !transitions.is_empty() {
            debug!(
                target = "runtime_orchestrator",
                count = transitions.len(),
                "health transitions observed"
            );
        }
        snapshot
    }

    /// Get (or lazily create) the cached adapter for `backend`.
    async fn ensure_adapter(
        &self,
        backend: RuntimeBackend,
    ) -> Result<Arc<dyn RuntimeAdapter>, RuntimeAdapterError> {
        // Fast path: read lock.
        {
            let read_guard = self.adapters.read().await;
            if let Some(existing) = read_guard.get(&backend) {
                return Ok(existing.clone());
            }
        }
        // Slow path: take write lock, double-check, instantiate.
        let mut write_guard = self.adapters.write().await;
        if let Some(existing) = write_guard.get(&backend) {
            return Ok(existing.clone());
        }
        let factory_cfg = {
            let cfg = self.config.read().await;
            cfg.factory_config_for(backend)
        };
        let adapter_box = create_runtime(factory_cfg)?;
        let arc: Arc<dyn RuntimeAdapter> = Arc::from(adapter_box);
        write_guard.insert(backend, arc.clone());
        Ok(arc)
    }
}

/// Convert `AgentRole` to its serde snake_case string. Used by the
/// `RuntimeEvent::Fallback` sink emission — the providers crate does
/// not depend on `elai-ipc-types`, so we ship the role as a string.
fn agent_role_to_snake(role: AgentRole) -> String {
    // Use serde to get the canonical snake_case form so the wire shape
    // matches the `SsbPayload::RuntimeFallback` payload's `role` field
    // exactly.
    serde_json::to_value(role)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| format!("{role:?}").to_lowercase())
}

/// Classify whether an adapter error should cause the orchestrator to
/// advance to the next fallback entry, or surface immediately.
fn is_recoverable(err: &RuntimeAdapterError) -> bool {
    matches!(
        err,
        RuntimeAdapterError::NotImplemented { .. }
            | RuntimeAdapterError::Network(_)
            | RuntimeAdapterError::HealthTimeout { .. }
            | RuntimeAdapterError::BinaryNotFound { .. }
            | RuntimeAdapterError::SubprocessSpawn { .. }
            | RuntimeAdapterError::StartupTimeout { .. }
            | RuntimeAdapterError::IdleTimeout { .. }
            | RuntimeAdapterError::ExitNonZero { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_providers::runtime::RuntimeFactoryConfig;

    fn cfg_with_primary(primary: RuntimeBackend) -> RuntimeConfig {
        let mut cfg = RuntimeConfig::default();
        for role in [
            AgentRole::Executor,
            AgentRole::Designer,
            AgentRole::Auditor,
        ] {
            cfg.per_role.set(role, primary);
        }
        cfg
    }

    #[tokio::test]
    async fn for_role_returns_primary_adapter_by_default() {
        let orch = RuntimeOrchestrator::new(cfg_with_primary(RuntimeBackend::Ollama));
        let adapter = orch
            .for_role(AgentRole::Executor)
            .await
            .expect("must return adapter (stubs are infallible)");
        // Ollama is a stub at B-37C library-tier; create_runtime returns a
        // boxed adapter without error.
        assert_eq!(adapter.backend(), RuntimeBackend::Ollama);
    }

    #[tokio::test]
    async fn for_role_falls_back_when_primary_factory_fails() {
        // Construct a config whose primary expects a missing field by
        // pointing AnthropicApi at an explicit RuntimeFactoryConfig that
        // lacks auth_source. AnthropicApi requires auth, so from_factory_config
        // returns MissingField; the orchestrator should fall through to
        // Sidecar (which is unconfigured but its adapter constructor is
        // infallible — health_check will report Unavailable).
        let mut cfg = RuntimeConfig::default();
        cfg.per_role
            .set(AgentRole::Executor, RuntimeBackend::AnthropicApi);
        cfg.fallback_chain = vec![RuntimeBackend::AnthropicApi, RuntimeBackend::Sidecar];

        // Override the AnthropicApi factory config to deliberately omit
        // auth_source. The constructor will fail.
        cfg.set_factory_config(
            RuntimeBackend::AnthropicApi,
            RuntimeFactoryConfig::for_backend(RuntimeBackend::AnthropicApi),
        );

        let orch = RuntimeOrchestrator::new(cfg);
        let adapter = orch
            .for_role(AgentRole::Executor)
            .await
            .expect("Sidecar fallback adapter must instantiate");
        assert_eq!(adapter.backend(), RuntimeBackend::Sidecar);
    }

    #[tokio::test]
    async fn try_complete_records_fallback_step_event() {
        // Set up a chain where the primary returns NotImplemented (stub
        // adapters: e.g. Ollama at B-37C library-tier). The orchestrator
        // should walk the chain and surface an AllBackendsFailed if every
        // entry is a stub (Sidecar without a server URL also returns
        // failure on complete).
        let mut cfg = RuntimeConfig::default();
        cfg.per_role.set(AgentRole::Executor, RuntimeBackend::Ollama);
        // Single-entry chain to keep the test deterministic.
        cfg.fallback_chain = vec![RuntimeBackend::Ollama];

        let orch = RuntimeOrchestrator::new(cfg);
        let result = orch
            .try_complete(
                AgentRole::Executor,
                vec![Message {
                    role: elai_providers::runtime::MessageRole::User,
                    content: "ping".into(),
                }],
                CompletionConfig::default(),
            )
            .await;

        // Should fail with AllBackendsFailed because Ollama is the only
        // entry and at B-37C it's a NotImplemented stub.
        match result {
            Err(RuntimeOrchestratorError::AllBackendsFailed { tried }) => {
                assert_eq!(tried, vec![RuntimeBackend::Ollama]);
            }
            other => panic!("expected AllBackendsFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn health_snapshot_covers_every_default_backend() {
        let orch = RuntimeOrchestrator::new(RuntimeConfig::default());
        let snapshot = orch.health_snapshot().await;
        assert_eq!(snapshot.len(), RuntimeBackend::all().len());
        for backend in RuntimeBackend::all() {
            assert!(
                snapshot.iter().any(|(b, _)| b == backend),
                "snapshot missing {backend:?}"
            );
        }
    }

    #[test]
    fn agent_role_to_snake_yields_canonical_form() {
        // The providers-side RuntimeEvent::Fallback carries `role` as a
        // string, and the orchestrator-side SsbPayload re-deserialises
        // it back into AgentRole — so the wire shape MUST match the
        // serde rename_all="snake_case" form. Verify the helper.
        assert_eq!(agent_role_to_snake(AgentRole::Executor), "executor");
        assert_eq!(agent_role_to_snake(AgentRole::ContextCompiler), "context_compiler");
        assert_eq!(
            agent_role_to_snake(AgentRole::AuditorBabyStepDetail),
            "auditor_baby_step_detail"
        );
    }

    #[tokio::test]
    async fn set_config_clears_adapter_cache() {
        let orch = RuntimeOrchestrator::new(RuntimeConfig::default());
        let _adapter = orch
            .for_role(AgentRole::Executor)
            .await
            .expect("initial adapter");
        // Cache populated; verify count==1.
        {
            let cache = orch.adapters.read().await;
            assert!(!cache.is_empty());
        }
        // Replace config; cache must be cleared.
        orch.set_config(RuntimeConfig::default()).await;
        let cache = orch.adapters.read().await;
        assert!(cache.is_empty());
    }
}
