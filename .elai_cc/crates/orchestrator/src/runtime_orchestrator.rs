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

use elai_core::clock::{ClockProvider, OsClock};
use elai_ipc_types::AgentRole;
use elai_providers::runtime::{
    backend_name_hash, cost_model_for, create_runtime, role_name_hash, runtime_event_sink,
    CompletionConfig, CompletionResponse, Message, RuntimeAdapter, RuntimeAdapterError,
    RuntimeBackend, RuntimeEvent, RuntimeHealth,
};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::handoff_payload::compress_handoff_payload;
use crate::role_predict::{predict_next_role, tier_allows_prewarm, VramTier};
use crate::role_prompt::{
    build_role_prompt, default_voice_toml_path, load_voice_anchor_from_toml, VoiceAnchor,
};
use crate::runtime_config::RuntimeConfig;
use crate::ssb::{
    BackendCostAccruedPayload, RuntimeHandoffEventKind, RuntimeHandoffEventPayload, Ssb, SsbPayload,
    ELAI_E3_RUNTIME_HANDOFF_ENABLE_ENV,
};

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
    /// E-3 sub-phase 4B — optional SSB writer threaded by the wiring
    /// layer (`AppState::writer` in the dashboard, `Ssb::new(...)`
    /// followed by `with_ssb` in the CLI). When `None`, the role-prompt
    /// wrap + pre-warm dispatch still fire but emission collapses to a
    /// `tracing::info` line (matches sub-phase 4A behaviour).
    ssb: Option<Arc<Ssb>>,
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
            ssb: None,
        }
    }

    /// E-3 sub-phase 4B — attach an SSB writer so `try_complete` can
    /// emit `E3RuntimeHandoffEvent::UnifiedPreambleApplied` /
    /// `RuntimePrewarm{Started,Complete}` rows. Builder-style to keep
    /// the 6+ existing `RuntimeOrchestrator::new` call sites (CLI
    /// `measure.rs`, `runtime_cmd.rs`, dashboard `AppState`, tests)
    /// untouched.
    #[must_use]
    pub fn with_ssb(mut self, ssb: Arc<Ssb>) -> Self {
        self.ssb = Some(ssb);
        self
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

        // E-3 sub-phase 4B: wrap `config.system` with the unified
        // role-prompt preamble BEFORE the fallback chain so every
        // backend in the chain sees the same anchor + calibration +
        // forbidden-opener block. Wire-dark unless
        // ELAI_E3_RUNTIME_HANDOFF_ENABLE=1; on env-off the wrap returns
        // the original system text unchanged (no behaviour change).
        // On env-on we load `.elai_cc/config/voice.toml`, build the
        // wrapped preamble, substitute config.system, and emit
        // E3RuntimeHandoffEvent::UnifiedPreambleApplied through the
        // optional Ssb handle attached via `with_ssb`.
        let config = self.wrap_system_with_role_prompt(role, config).await;

        // E-3 sub-phase 4B: compress the rolling chat history BEFORE the
        // fallback chain so every backend in the chain (and any later
        // cloud->local handoff) sees one constant-size payload. Wire-dark
        // unless ELAI_E3_RUNTIME_HANDOFF_ENABLE=1; on env-off the helper
        // returns the messages untouched (no behaviour change). On env-on
        // it runs the deterministic `compress_handoff_payload` heuristic
        // and, when the payload actually shrank, replaces `messages` with
        // the compressed list AND emits an
        // E3RuntimeHandoffEvent::HandoffPayloadCompressed row through the
        // optional Ssb handle attached via `with_ssb`. This is the sole
        // PRODUCTION caller of `compress_handoff_payload` (previously the
        // function had only `#[cfg(test)]` callers — R6/R7 wire-dark).
        let messages = self.compress_handoff_for_dispatch(role, messages).await;

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
                    // E-3 sub-phase 4B: tier-gated pre-warm of the
                    // predicted-next backend. Spawned on the current
                    // tokio runtime so the response returns immediately;
                    // pre-warm failures are logged but never surface to
                    // the caller. Gated by ELAI_E3_RUNTIME_HANDOFF_ENABLE
                    // + tier check inside `predict_and_warm_next_role`.
                    self.predict_and_warm_next_role(role).await;
                    // C-44 — emit the BackendCostAccrued off-agent artifact after
                    // a successful PAID-backend invocation. `cost_model_for`
                    // returns None for local/free backends (Sidecar, Mlx, ...),
                    // so no row is written for zero-marginal-cost calls; the
                    // Composer 2 (and future paid clouds) path computes
                    // est_cost_micros from the provider-reported token usage via
                    // the source-grounded BackendCostModel (arXiv:2603.24477
                    // grounds Composer 2 identity; pricing is from cursor.com/docs
                    // per cost.rs). This is the sole production emit site for the
                    // BackendCostAccrued variant (previously declared but never
                    // emitted from a real invoke — wire-dark).
                    if let (Some(ssb), Some(model)) =
                        (self.ssb.as_ref(), cost_model_for(backend))
                    {
                        let (tokens_in, tokens_out) = response.usage;
                        let est_cost_micros = model.estimate_micros(tokens_in, tokens_out);
                        let payload = SsbPayload::BackendCostAccrued(BackendCostAccruedPayload {
                            backend_name_hash: backend_name_hash(backend),
                            role_name_hash: role_name_hash(role),
                            tokens_in,
                            tokens_out,
                            est_cost_micros,
                            ts_ms: u64::try_from(now_ms()).unwrap_or(0),
                        });
                        if let Err(e) = ssb
                            .write_payload("runtime_orchestrator", 0, payload, Vec::new())
                            .await
                        {
                            warn!(
                                target = "runtime_orchestrator",
                                error = %e,
                                "ssb write_payload failed for backend_cost_accrued"
                            );
                        }
                    } else if let Some(ssb) = self.ssb.as_ref() {
                        // P0-3 — local backends have zero marginal API cost.
                        // Emit BackendCostAccrued with est_cost_micros=0 so
                        // the H-9 cockpit and E-39 CostPanel see local-lane
                        // activity. Gate on tokens_in > 0 || tokens_out > 0
                        // to suppress the sidecar parse-error (0,0,0) noise
                        // rows (sidecar.rs:249 returns usage=(0,0) on error).
                        let (tokens_in, tokens_out) = response.usage;
                        if backend.is_local() && (tokens_in > 0 || tokens_out > 0) {
                            let payload =
                                SsbPayload::BackendCostAccrued(BackendCostAccruedPayload {
                                    backend_name_hash: backend_name_hash(backend),
                                    role_name_hash: role_name_hash(role),
                                    tokens_in,
                                    tokens_out,
                                    est_cost_micros: 0,
                                    ts_ms: u64::try_from(now_ms()).unwrap_or(0),
                                });
                            if let Err(e) = ssb
                                .write_payload(
                                    "runtime_orchestrator",
                                    0,
                                    payload,
                                    Vec::new(),
                                )
                                .await
                            {
                                warn!(
                                    target = "runtime_orchestrator",
                                    error = %e,
                                    "ssb write_payload failed for local backend_cost_accrued"
                                );
                            }
                        }
                    }
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

impl RuntimeOrchestrator {
    /// E-3 sub-phase 4B: wrap `config.system` with the unified
    /// role-prompt preamble loaded from `.elai_cc/config/voice.toml`,
    /// emitting `E3RuntimeHandoffEvent::UnifiedPreambleApplied` through
    /// the optional Ssb handle when present.
    ///
    /// Behaviour matrix:
    ///   * `ELAI_E3_RUNTIME_HANDOFF_ENABLE` not set / falsy: returns
    ///     `config` unchanged (wire-dark default; no behaviour change
    ///     vs pre-E-3).
    ///   * `ELAI_E3_RUNTIME_HANDOFF_ENABLE=1` and voice.toml load
    ///     succeeds: replaces `config.system` with
    ///     `build_role_prompt(role, base, anchor)`. When the
    ///     orchestrator was constructed via `with_ssb` the wrap also
    ///     emits a `E3RuntimeHandoffEvent::UnifiedPreambleApplied` row;
    ///     otherwise it falls back to the 4A tracing-line surface.
    ///   * `ELAI_E3_RUNTIME_HANDOFF_ENABLE=1` and voice.toml load
    ///     fails: logs a `warn` and returns `config` unchanged
    ///     (wire-dark fail-soft so a missing toml never blocks a
    ///     dispatch).
    ///
    /// Idempotent at the per-call level: the wrap reads env / disk
    /// once per `try_complete` invocation and applies the preamble to
    /// whichever `config.system` was passed in.
    async fn wrap_system_with_role_prompt(
        &self,
        role: AgentRole,
        mut config: CompletionConfig,
    ) -> CompletionConfig {
        if !is_e3_runtime_handoff_enabled() {
            return config;
        }
        let voice_role = agent_role_to_voice_section(role);
        let anchor = match load_voice_anchor_for_role(&voice_role) {
            Some(a) => a,
            None => return config,
        };
        if anchor.is_empty() {
            return config;
        }
        let base = config.system.clone().unwrap_or_default();
        let wrapped = build_role_prompt(&voice_role, &base, &anchor);
        let preamble_len = wrapped.len();
        let base_prompt_len = base.len();
        // Blake3 short hash of the wrapped preamble — stable across
        // dispatches that share the same voice anchor + base prompt,
        // so dashboards can render "single voice anchor blake3 across
        // all 3 dispatches in this cycle" per the E-3 acceptance gate.
        let preamble_blake3 = blake3::hash(wrapped.as_bytes()).to_hex().to_string();
        let preamble_blake3 = preamble_blake3[..16].to_owned();

        // Tracing log fires regardless of Ssb availability so log
        // scrapers built against 4A keep working. Fields stay in lockstep
        // with RuntimeHandoffEventPayload so the scraper can promote
        // them to the SSB row shape transparently.
        info!(
            target = "runtime_orchestrator",
            e3_runtime_handoff_event = "unified_preamble_applied",
            ?role,
            voice_section = %voice_role,
            base_prompt_len,
            preamble_len,
            preamble_blake3 = %preamble_blake3,
            "build_role_prompt wrapped system message with voice anchor"
        );
        // Emit the SSB row when we have an Ssb handle. Failures during
        // emission collapse to a `warn` so a transient writer error
        // never blocks the underlying dispatch.
        if let Some(ssb) = self.ssb.as_ref() {
            let payload = SsbPayload::E3RuntimeHandoffEvent(RuntimeHandoffEventPayload {
                kind: RuntimeHandoffEventKind::UnifiedPreambleApplied,
                role: Some(agent_role_to_snake(role)),
                backend: None,
                session_id: Some(ssb.session_id().to_owned()),
                details_json: serde_json::json!({
                    "voice_section": voice_role,
                    "base_prompt_len": base_prompt_len,
                    "preamble_len": preamble_len,
                    "preamble_blake3": preamble_blake3,
                }),
                ts_ms: now_ms(),
            });
            if let Err(e) = ssb
                .write_payload("runtime_orchestrator", 0, payload, Vec::new())
                .await
            {
                warn!(
                    target = "runtime_orchestrator",
                    error = %e,
                    "ssb write_payload failed for unified_preamble_applied"
                );
            }
        }
        config.system = Some(wrapped);
        config
    }

    /// E-3 sub-phase 4B: tier-gated pre-warm of the predicted-next
    /// backend. After a successful `try_complete` dispatch, look up
    /// the next role likely to fire (per `role_predict::predict_next_role`),
    /// resolve its primary backend through `for_role` (lazy cache
    /// hit if already instantiated), and call `adapter.warm()`. On
    /// 12GB / Low tier this is a no-op; on 24GB+ we issue a 1-token
    /// CUDA-graph / connection-pool warm. Wire-dark unless
    /// ELAI_E3_RUNTIME_HANDOFF_ENABLE=1.
    async fn predict_and_warm_next_role(&self, current_role: AgentRole) {
        if !is_e3_runtime_handoff_enabled() {
            return;
        }
        let tier = VramTier::from_env();
        if !tier_allows_prewarm(tier) {
            debug!(
                target = "runtime_orchestrator",
                tier = ?tier,
                "predict_and_warm_next_role: tier disallows pre-warm; skip"
            );
            return;
        }
        let next_role = match predict_next_role(current_role) {
            Some(r) => r,
            None => return,
        };
        // Resolve the predicted-next backend without consuming the
        // orchestrator's fallback chain — pre-warm should fail soft.
        let adapter = match self.for_role(next_role).await {
            Ok(a) => a,
            Err(e) => {
                debug!(
                    target = "runtime_orchestrator",
                    ?next_role,
                    error = %e,
                    "predict_and_warm_next_role: for_role failed; skip"
                );
                return;
            }
        };
        let backend = adapter.backend();
        let backend_str = format!("{backend:?}").to_lowercase();

        // Emit RuntimePrewarmStarted before the warm call so the
        // dashboard can render the in-flight state. Same fail-soft
        // pattern as the preamble emission above.
        if let Some(ssb) = self.ssb.as_ref() {
            let payload = SsbPayload::E3RuntimeHandoffEvent(RuntimeHandoffEventPayload {
                kind: RuntimeHandoffEventKind::RuntimePrewarmStarted,
                role: Some(agent_role_to_snake(next_role)),
                backend: Some(backend_str.clone()),
                session_id: Some(ssb.session_id().to_owned()),
                details_json: serde_json::json!({
                    "predicted_role": agent_role_to_snake(next_role),
                    "backend": backend_str,
                    "vram_tier": tier.as_str(),
                }),
                ts_ms: now_ms(),
            });
            if let Err(e) = ssb
                .write_payload("runtime_orchestrator", 0, payload, Vec::new())
                .await
            {
                warn!(
                    target = "runtime_orchestrator",
                    error = %e,
                    "ssb write_payload failed for runtime_prewarm_started"
                );
            }
        }

        let started = OsClock.instant_now();
        let warm_result = adapter.warm().await;
        let duration_ms = i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX);
        let ok = warm_result.is_ok();
        if let Err(ref e) = warm_result {
            debug!(
                target = "runtime_orchestrator",
                ?next_role,
                ?backend,
                duration_ms,
                error = %e,
                "predict_and_warm_next_role: warm() failed (non-blocking)"
            );
        }

        if let Some(ssb) = self.ssb.as_ref() {
            let payload = SsbPayload::E3RuntimeHandoffEvent(RuntimeHandoffEventPayload {
                kind: RuntimeHandoffEventKind::RuntimePrewarmComplete,
                role: Some(agent_role_to_snake(next_role)),
                backend: Some(backend_str.clone()),
                session_id: Some(ssb.session_id().to_owned()),
                details_json: serde_json::json!({
                    "backend": backend_str,
                    "duration_ms": duration_ms,
                    "ok": ok,
                    "cuda_graph_captured": false,
                }),
                ts_ms: now_ms(),
            });
            if let Err(e) = ssb
                .write_payload("runtime_orchestrator", 0, payload, Vec::new())
                .await
            {
                warn!(
                    target = "runtime_orchestrator",
                    error = %e,
                    "ssb write_payload failed for runtime_prewarm_complete"
                );
            }
        }
    }

    /// E-3 sub-phase 4B: compress the inbound rolling chat history to a
    /// constant-size budget before it is handed to the fallback chain
    /// (and, transitively, across a cloud<->local PerRoleRuntime swap).
    ///
    /// This is the production wiring of
    /// [`crate::handoff_payload::compress_handoff_payload`]: until now the
    /// compressor had only `#[cfg(test)]` callers, leaving it a
    /// producer-without-consumer (R6) / no-non-test-reachability (R7)
    /// wire-dark deliverable per the E-3 plan-status punch list.
    ///
    /// Behaviour matrix:
    ///   * `ELAI_E3_RUNTIME_HANDOFF_ENABLE` not set / falsy: returns
    ///     `messages` unchanged (wire-dark default; the compressor is not
    ///     invoked at all, so there is zero behaviour change vs pre-E-3).
    ///   * `ELAI_E3_RUNTIME_HANDOFF_ENABLE=1` and the payload already fits
    ///     the budget: `compress_handoff_payload` returns the messages
    ///     verbatim, `compressed_tokens == original_tokens`, NO SSB row is
    ///     emitted, and the original `messages` are returned (the cheap
    ///     pass-through path — emitting a "compressed 1x" row would be
    ///     noise).
    ///   * `ELAI_E3_RUNTIME_HANDOFF_ENABLE=1` and the payload was actually
    ///     trimmed (`compressed_tokens < original_tokens`): emits a
    ///     `E3RuntimeHandoffEvent::HandoffPayloadCompressed` row carrying
    ///     `{original_tokens, compressed_tokens, ratio}` through the
    ///     optional `Ssb` handle, and returns the COMPRESSED message list
    ///     so the downstream `adapter.complete` sees the smaller payload.
    ///
    /// Fail-soft: an SSB write error is logged at `warn` and never blocks
    /// the dispatch (mirrors the preamble / pre-warm emission paths).
    async fn compress_handoff_for_dispatch(
        &self,
        role: AgentRole,
        messages: Vec<Message>,
    ) -> Vec<Message> {
        if !is_e3_runtime_handoff_enabled() {
            return messages;
        }
        let target_tokens = handoff_target_tokens();
        let compressed = compress_handoff_payload(&messages, target_tokens);

        // Cheap path: nothing was trimmed. Return the original messages and
        // emit no row — the compressor is a pass-through under budget.
        if compressed.compressed_tokens >= compressed.original_tokens {
            return messages;
        }

        // Real compression happened. Log + (optionally) emit the off-agent
        // SSB row, then hand the compressed payload downstream.
        info!(
            target = "runtime_orchestrator",
            e3_runtime_handoff_event = "handoff_payload_compressed",
            ?role,
            original_tokens = compressed.original_tokens,
            compressed_tokens = compressed.compressed_tokens,
            target_tokens,
            ratio = compressed.ratio,
            "compress_handoff_payload trimmed rolling chat history for dispatch"
        );
        if let Some(ssb) = self.ssb.as_ref() {
            let payload = SsbPayload::E3RuntimeHandoffEvent(RuntimeHandoffEventPayload {
                kind: RuntimeHandoffEventKind::HandoffPayloadCompressed,
                role: Some(agent_role_to_snake(role)),
                backend: None,
                session_id: Some(ssb.session_id().to_owned()),
                details_json: serde_json::json!({
                    "original_tokens": compressed.original_tokens,
                    "compressed_tokens": compressed.compressed_tokens,
                    "ratio": compressed.ratio,
                    "target_tokens": target_tokens,
                }),
                ts_ms: now_ms(),
            });
            if let Err(e) = ssb
                .write_payload("runtime_orchestrator", 0, payload, Vec::new())
                .await
            {
                warn!(
                    target = "runtime_orchestrator",
                    error = %e,
                    "ssb write_payload failed for handoff_payload_compressed"
                );
            }
        }
        compressed.messages
    }
}

/// Current wall-clock millis since UNIX_EPOCH. Used by the E-3 SSB
/// payloads so the row's wall-clock timestamp matches the moment of
/// emission rather than the SSB writer's commit time.
fn now_ms() -> i64 {
    i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX)
}

/// Token budget the handoff payload is compressed down to before a
/// dispatch. Reads `ELAI_LOCAL_MAX_CONTEXT` (the documented hard ceiling
/// for the local sidecar window per CLAUDE.md) so the compressed payload
/// is sized for the SMALLEST receiver in the PerRoleRuntime mix (the
/// local Qwen). Falls back to a conservative 25_000-token default that
/// matches the CLAUDE.md "use 25000-28000 in production for headroom"
/// guidance. A parse failure or zero value also falls back to the
/// default so a malformed env never collapses the budget to 0 (which
/// `compress_handoff_payload` would treat as "elide all history").
const HANDOFF_DEFAULT_TARGET_TOKENS: usize = 25_000;

fn handoff_target_tokens() -> usize {
    match std::env::var("ELAI_LOCAL_MAX_CONTEXT") {
        Ok(v) => match v.trim().parse::<usize>() {
            Ok(n) if n > 0 => n,
            _ => HANDOFF_DEFAULT_TARGET_TOKENS,
        },
        Err(_) => HANDOFF_DEFAULT_TARGET_TOKENS,
    }
}

/// Resolve the env-gate truthiness for E-3 runtime handoff. Accepts
/// `"1"` / `"true"` / `"yes"` / `"on"` (case-insensitive). Mirrors
/// the precedent set by other E-N wire-dark gates.
fn is_e3_runtime_handoff_enabled() -> bool {
    match std::env::var(ELAI_E3_RUNTIME_HANDOFF_ENABLE_ENV) {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => false,
    }
}

/// Map an [`AgentRole`] to the voice.toml section key. The toml ships
/// `default` / `planner` / `coder` / `critic` sections; auditor-shape
/// roles route to `critic`, executor-shape roles route to `coder`,
/// designer / prompt-engineer / planner-shape roles route to `planner`,
/// everything else falls through to `default` and load_voice_anchor's
/// own fallback chain picks it up.
fn agent_role_to_voice_section(role: AgentRole) -> String {
    match role {
        AgentRole::Executor => "coder".to_owned(),
        AgentRole::Auditor
        | AgentRole::AuditorConcepts
        | AgentRole::AuditorPhases
        | AgentRole::AuditorSteps
        | AgentRole::AuditorBabySteps
        | AgentRole::AuditorBabyStepDetail
        | AgentRole::AuditorProductionGrade
        | AgentRole::IdeaCritic
        | AgentRole::InjectionClassifier => "critic".to_owned(),
        AgentRole::Designer
        | AgentRole::PromptEngineer
        | AgentRole::IdeaFramer => "planner".to_owned(),
        _ => "default".to_owned(),
    }
}

/// Load the voice anchor for `voice_section`. Returns `None` on any
/// load error (with a `warn` log) so the orchestrator stays fail-soft;
/// returning `Some(VoiceAnchor::empty())` is also safe and short-circuits
/// the wrap.
fn load_voice_anchor_for_role(voice_section: &str) -> Option<VoiceAnchor> {
    let path = default_voice_toml_path();
    match load_voice_anchor_from_toml(voice_section, &path) {
        Ok(anchor) => Some(anchor),
        Err(err) => {
            warn!(
                target = "runtime_orchestrator",
                voice_section,
                error = %err,
                path = %path.display(),
                "voice.toml load failed; falling back to unwrapped system prompt"
            );
            None
        }
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

    // -----------------------------------------------------------------
    // E-3 sub-phase 4B — compress_handoff_payload production-wiring smoke.
    //
    // Off-agent artifact proof: drive the REAL production dispatch method
    // `try_complete` (NOT the pure `compress_handoff_payload` fn directly)
    // with an oversized rolling history under ELAI_E3_RUNTIME_HANDOFF_ENABLE=1,
    // then read the `handoff_payload_compressed` SSB row back out of a temp
    // Ssb via `SsbReader::query_by_type`. The row is written by the
    // orchestrator's own emit path (the agent does NOT author it), so it is
    // a genuine off-agent artifact per wiring-enforcement R4.
    //
    // R7 reachability: `try_complete` is a `pub` method on a
    // `RuntimeOrchestrator` that production constructs WITH an Ssb handle
    // (dashboard-v2/src-tauri/src/state.rs `RuntimeOrchestrator::new(...).
    // with_ssb(..)`); this smoke exercises that exact construction shape.
    //
    // SINGLE sequential #[tokio::test]: ELAI_E3_RUNTIME_HANDOFF_ENABLE is a
    // process-global env var that would race a parallel test split, so this
    // is the only test in this module that sets it, and it restores it on
    // exit.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn try_complete_emits_handoff_payload_compressed_row_off_agent() {
        use crate::ssb::{Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader};
        use elai_providers::runtime::MessageRole;

        // --- env: enable the gate + pin a small budget so compression
        //     definitely fires on the oversized fixture. ---
        let prev_enable = std::env::var(ELAI_E3_RUNTIME_HANDOFF_ENABLE_ENV).ok();
        let prev_ctx = std::env::var("ELAI_LOCAL_MAX_CONTEXT").ok();
        std::env::set_var(ELAI_E3_RUNTIME_HANDOFF_ENABLE_ENV, "1");
        // 200-token budget — the fixture is far larger so the heuristic
        // must trim it.
        std::env::set_var("ELAI_LOCAL_MAX_CONTEXT", "200");

        let dir = tempfile::TempDir::new().expect("tempdir");
        let ssb = Arc::new(
            Ssb::new(
                SsbConfig {
                    db_path: dir.path().join("e3_handoff_smoke.db"),
                    pool_size: Some(2),
                    project_id: "test-project".into(),
                    session_id: "s-e3-4b-smoke".into(),
                },
                Arc::new(OsClock),
            )
            .expect("ssb init"),
        );

        // Construct the orchestrator the SAME way production does: a real
        // config + an attached Ssb writer.
        let mut cfg = RuntimeConfig::default();
        cfg.per_role.set(AgentRole::Executor, RuntimeBackend::Ollama);
        cfg.fallback_chain = vec![RuntimeBackend::Ollama];
        let orch = RuntimeOrchestrator::new(cfg).with_ssb(Arc::clone(&ssb));

        // Oversized rolling history: 40 fat head messages + a short tail.
        let mut messages: Vec<Message> = Vec::new();
        for i in 0..40 {
            messages.push(Message {
                role: MessageRole::User,
                content: "a".repeat(2_000 + i),
            });
        }
        for i in 0..4 {
            messages.push(Message {
                role: MessageRole::Assistant,
                content: format!("tail-{i}"),
            });
        }

        // Drive the REAL production dispatch path. The Ollama stub fails on
        // `complete`, so `try_complete` returns AllBackendsFailed — but the
        // compression + SSB emission run BEFORE the fallback loop, so the
        // off-agent row is written regardless of dispatch outcome.
        let _ = orch
            .try_complete(AgentRole::Executor, messages, CompletionConfig::default())
            .await;

        // --- read the off-agent artifact back ---
        let rows = ssb
            .query_by_type(SsbActionType::E3RuntimeHandoffEvent)
            .expect("query_by_type");
        let mut found_compressed = false;
        for entry in &rows {
            if let Ok(SsbPayload::E3RuntimeHandoffEvent(p)) =
                rmp_serde::from_slice::<SsbPayload>(&entry.payload)
            {
                if p.kind == RuntimeHandoffEventKind::HandoffPayloadCompressed {
                    found_compressed = true;
                    let orig = p.details_json["original_tokens"].as_u64().unwrap_or(0);
                    let comp = p.details_json["compressed_tokens"].as_u64().unwrap_or(u64::MAX);
                    let ratio = p.details_json["ratio"].as_f64().unwrap_or(0.0);
                    assert!(
                        comp < orig,
                        "compressed_tokens ({comp}) must be < original_tokens ({orig})"
                    );
                    assert!(ratio > 1.0, "ratio ({ratio}) must be > 1.0 after a real trim");
                    assert_eq!(
                        p.role.as_deref(),
                        Some("executor"),
                        "row must carry the dispatch role"
                    );
                }
            }
        }
        assert!(
            found_compressed,
            "expected a handoff_payload_compressed off-agent SSB row, found none in {} e3 rows",
            rows.len()
        );

        // --- restore env so peer tests are unaffected ---
        match prev_enable {
            Some(v) => std::env::set_var(ELAI_E3_RUNTIME_HANDOFF_ENABLE_ENV, v),
            None => std::env::remove_var(ELAI_E3_RUNTIME_HANDOFF_ENABLE_ENV),
        }
        match prev_ctx {
            Some(v) => std::env::set_var("ELAI_LOCAL_MAX_CONTEXT", v),
            None => std::env::remove_var("ELAI_LOCAL_MAX_CONTEXT"),
        }
    }
}
