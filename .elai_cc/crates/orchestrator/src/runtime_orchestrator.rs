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
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use elai_core::clock::{ClockProvider, OsClock};
use elai_ipc_types::AgentRole;
use elai_providers::runtime::{
    apply_trust_marker, backend_name_hash, cost_model_for, create_runtime, role_name_hash,
    runtime_event_sink, CompletionConfig, CompletionResponse, Message, RuntimeAdapter,
    RuntimeAdapterError, RuntimeBackend, RuntimeEvent, RuntimeHealth,
};
use elai_providers::{default_manifest_path, DecodingMode};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::cloud_mode::{
    CloudConfig, CloudModeRouter, ExecMode, RoutedProvider, SpendSnapshotInput,
};
use crate::grammar_gate::call_grammar_gate;
use crate::handoff_payload::compress_handoff_payload;
use crate::local_serving::{
    DecideInput, LocalServingDecision, LocalServingRouter, TierResidencySnapshot, TierState,
};
use crate::role_predict::{predict_next_role, tier_allows_prewarm, VramTier};
use crate::seat_registry::{SeatAdmitDecision, SeatRegistry};
use crate::role_prompt::{
    build_role_prompt, default_voice_toml_path, load_voice_anchor_from_toml, VoiceAnchor,
};
use crate::runtime_config::RuntimeConfig;
use crate::ssb::{
    BackendCostAccruedPayload, RouteDecisionEventKind, RouteDecisionEventPayload,
    RuntimeHandoffEventKind, RuntimeHandoffEventPayload, Ssb, SsbPayload, UsageTurnEventKind,
    UsageTurnEventPayload, ELAI_E3_RUNTIME_HANDOFF_ENABLE_ENV,
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
    /// J-30 T7a: a mid-sequence backend swap was refused. Once a backend
    /// has been resolved for a given `turn_id`, the orchestrator pins that
    /// backend until `release_turn_lock(turn_id)` is called; any
    /// `for_role`/`try_complete` call with the same `turn_id` that would
    /// resolve a DIFFERENT backend returns this error. The
    /// `EscalateToCloud` promotion path is orthogonal (it does not call
    /// `for_role`/`try_complete` for the cloud leg) and is never blocked
    /// by this lock.
    #[error("mid-sequence backend swap refused: locked {locked:?}, attempted {attempted:?}, turn_id {turn_id}")]
    BackendLockedMidSequence {
        locked: RuntimeBackend,
        attempted: RuntimeBackend,
        turn_id: u64,
    },
}

/// J-30 T7a: pin stamped on `RuntimeOrchestrator::turn_lock` once a
/// backend has been resolved for a turn. The pin is cleared by
/// `release_turn_lock(turn_id)` at sequence end. `turn_id == 0` is the
/// pre-warm sentinel: `for_role`/`try_complete` calls with `turn_id == 0`
/// NEVER stamp or check the lock (pre-warm is a prediction, not a
/// sequence commitment).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurnBackendPin {
    pub backend: RuntimeBackend,
    pub turn_id: u64,
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
    /// J-30 T7a: turn-scoped backend pin. Once a backend is resolved for
    /// a `turn_id != 0`, the pin refuses mid-sequence swap to a different
    /// backend. `turn_id == 0` is the pre-warm sentinel and never stamps
    /// the lock. Cleared by `release_turn_lock(turn_id)` at sequence end.
    turn_lock: RwLock<Option<TurnBackendPin>>,
    /// J-48: per-tier residency tracker (warm/cold + inflight count).
    /// Drives the `LocalServingRouter::decide()` pre-step in `try_complete`.
    tier_residency: RwLock<TierResidencySnapshot>,
    /// J-49: per-seat admission registry (fair-share / concurrency / spend).
    /// Drives the seat-admit pre-filter in `try_complete` BEFORE the J-48
    /// serving decision. Default [`SeatRegistry::single_local`] = one
    /// permissive "local" seat = always Admit (pre-J-49 behavior).
    seat_registry: RwLock<SeatRegistry>,
    /// J-49: the `UsageTurnEvent.identity` key for this orchestrator
    /// instance. Multi-seat callers override via [`Self::with_seat_identity`].
    seat_identity: String,
}

impl RuntimeOrchestrator {
    /// Build a fresh orchestrator from the given config. Does not
    /// instantiate any adapter — they are created lazily on first use.
    ///
    /// J-49: the seat registry is loaded from the path in `ELAI_SEATS_TOML`
    /// when set (degrading to `single_local` on parse/IO failure), else
    /// `single_local`. The seat identity is taken from `ELAI_SEAT_IDENTITY`
    /// when set, else the implicit `"local"` seat. The env-loaded values are
    /// applied via [`Self::with_seat_registry`] / [`Self::with_seat_identity`]
    /// so the env-driven path and the builder path share the same wiring.
    #[must_use]
    pub fn new(config: RuntimeConfig) -> Self {
        let seat_registry = match std::env::var_os("ELAI_SEATS_TOML") {
            Some(p) => SeatRegistry::load_from_path(std::path::Path::new(&p))
                .unwrap_or_else(|_| SeatRegistry::single_local()),
            None => SeatRegistry::single_local(),
        };
        let seat_identity = std::env::var("ELAI_SEAT_IDENTITY")
            .unwrap_or_else(|_| crate::seat_registry::LOCAL_SEAT_ID.to_string());
        Self {
            config: RwLock::new(config),
            adapters: RwLock::new(BTreeMap::new()),
            health: RwLock::new(BTreeMap::new()),
            ssb: None,
            turn_lock: RwLock::new(None),
            tier_residency: RwLock::new(TierResidencySnapshot::new()),
            seat_registry: RwLock::new(SeatRegistry::single_local()),
            seat_identity: crate::seat_registry::LOCAL_SEAT_ID.to_string(),
        }
        .with_seat_registry(seat_registry)
        .with_seat_identity(seat_identity)
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

    /// J-49: replace the default single-local seat registry with a
    /// multi-seat registry (loaded from `seats.toml` or constructed
    /// programmatically). The registry drives the per-seat admission
    /// pre-filter in `try_complete`.
    #[must_use]
    pub fn with_seat_registry(mut self, reg: SeatRegistry) -> Self {
        self.seat_registry = RwLock::new(reg);
        self
    }

    /// J-49: set the seat identity (the `UsageTurnEvent.identity` key)
    /// this orchestrator instance speaks as. Multi-seat callers use this
    /// so the admission registry can apply per-seat caps. Default
    /// `"local"`.
    #[must_use]
    pub fn with_seat_identity(mut self, id: impl Into<String>) -> Self {
        self.seat_identity = id.into();
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
    ///
    /// J-30 T7a: `turn_id` scopes the backend pin. `turn_id == 0` is the
    /// pre-warm sentinel: the lock is neither checked nor stamped (pre-warm
    /// is a prediction, not a sequence commitment). For `turn_id != 0`,
    /// once a backend is successfully resolved the pin is stamped on
    /// `turn_lock`; a subsequent call with the same `turn_id` that would
    /// resolve a DIFFERENT backend returns `BackendLockedMidSequence`.
    /// The pin is cleared by `release_turn_lock(turn_id)` at sequence end.
    pub async fn for_role(
        &self,
        role: AgentRole,
        turn_id: u64,
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

        // J-30 T7a: check the turn-scoped pin BEFORE the chain walk. If a
        // pin is already stamped for this turn_id and the primary (the
        // first backend the chain would try) differs from the pinned
        // backend, refuse mid-sequence swap. The check is skipped for
        // turn_id == 0 (pre-warm sentinel).
        if turn_id != 0 {
            let lock_guard = self.turn_lock.read().await;
            if let Some(pin) = *lock_guard {
                if pin.turn_id == turn_id && pin.backend != primary {
                    warn!(
                        target = "runtime_orchestrator",
                        locked = ?pin.backend,
                        attempted = ?primary,
                        turn_id,
                        "mid-sequence backend swap refused"
                    );
                    drop(lock_guard);
                    return Err(RuntimeOrchestratorError::BackendLockedMidSequence {
                        locked: pin.backend,
                        attempted: primary,
                        turn_id,
                    });
                }
            }
        }

        let mut last_err: Option<RuntimeAdapterError> = None;
        for backend in chain {
            tried.push(backend);
            match self.ensure_adapter(backend).await {
                Ok(adapter) => {
                    // J-30 T7a: stamp the turn-scoped pin on the first
                    // successful ensure_adapter in the chain. turn_id == 0
                    // (pre-warm sentinel) never stamps.
                    if turn_id != 0 {
                        let mut lock_guard = self.turn_lock.write().await;
                        match *lock_guard {
                            Some(pin) if pin.turn_id == turn_id => {
                                // Same turn: pin already stamped, no-op.
                            }
                            _ => {
                                *lock_guard = Some(TurnBackendPin {
                                    backend,
                                    turn_id,
                                });
                            }
                        }
                    }
                    return Ok(adapter);
                }
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

    pub async fn release_turn_lock(&self, turn_id: u64) {
        let mut lock_guard = self.turn_lock.write().await;
        if let Some(pin) = *lock_guard {
            if pin.turn_id == turn_id {
                *lock_guard = None;
            }
        }
    }

    /// J-48: decrement the inflight count for a tier after the chain
    /// walk completes (success or failure). Paired with the inflight
    /// bump in `try_complete` when ServeLocal/HibernateThenServe was
    /// chosen.
    async fn release_tier_inflight(&self, tier: VramTier) {
        let mut guard = self.tier_residency.write().await;
        if let Some(state) = guard.get_mut(&tier) {
            state.inflight = state.inflight.saturating_sub(1);
        }
    }

    /// J-49: release the seat's concurrency slot after the chain walk
    /// completes (success or failure). Paired with the `admit` call in
    /// `try_complete`. Only called when the seat was Admitted (Deny
    /// never increments inflight; Queued never increments inflight).
    async fn release_seat(&self) {
        let mut guard = self.seat_registry.write().await;
        guard.release(&self.seat_identity);
    }

    /// Drive a `complete` call for `role` with full fallback. On each
    /// attempt that fails with `Unavailable` / `NotImplemented` /
    /// `HealthTimeout` / `Network`, the chain advances. Returns the
    /// response plus the lifecycle event trail for the SSB writer.
    ///
    /// J-30 T7a: `turn_id` scopes the backend pin (see `for_role`'s
    /// doc comment). `turn_id == 0` is the pre-warm sentinel and never
    /// stamps or checks the lock.
    pub async fn try_complete(
        &self,
        role: AgentRole,
        messages: Vec<Message>,
        config: CompletionConfig,
        turn_id: u64,
    ) -> Result<CompleteOutcome, RuntimeOrchestratorError> {
        let cfg = self.config.read().await;
        let primary = cfg.per_role.get(role);
        let chain: Vec<RuntimeBackend> = std::iter::once(primary)
            .chain(cfg.fallback_chain.iter().copied().filter(|b| *b != primary))
            .collect();
        // I-34: snapshot grammar_manifest_path before releasing the config lock.
        let grammar_manifest_path: Option<PathBuf> = cfg.grammar_manifest_path.clone();
        drop(cfg);

        // J-30 T7a: check the turn-scoped pin BEFORE the chain walk.
        if turn_id != 0 {
            let lock_guard = self.turn_lock.read().await;
            if let Some(pin) = *lock_guard {
                if pin.turn_id == turn_id && pin.backend != primary {
                    warn!(
                        target = "runtime_orchestrator",
                        locked = ?pin.backend,
                        attempted = ?primary,
                        turn_id,
                        "mid-sequence backend swap refused"
                    );
                    drop(lock_guard);
                    return Err(RuntimeOrchestratorError::BackendLockedMidSequence {
                        locked: pin.backend,
                        attempted: primary,
                        turn_id,
                    });
                }
            }
        }

        // J-49: per-seat admission pre-filter (fair-share / concurrency /
        // spend). Runs BEFORE the J-48 serving decision so a denied/queued
        // seat never reaches the local serving router. Absent multi-seat
        // data = single "local" seat = always Admit (pre-J-49 behavior).
        // est_cost_micros is 0 here because the real cost is not known
        // until after a backend succeeds; spend caps therefore apply to
        // the turn count / concurrency dimension, not a pre-priced estimate.
        let seat_decision = {
            let mut seats = self.seat_registry.write().await;
            seats.admit(&self.seat_identity, 0)
        };
        let seat_admitted = matches!(seat_decision, SeatAdmitDecision::Admit);
        let seat_block_reason: Option<String> = match &seat_decision {
            SeatAdmitDecision::Admit => None,
            SeatAdmitDecision::Deny { reason } => Some(reason.clone()),
            SeatAdmitDecision::Queued { reason } => Some(reason.clone()),
        };

        // J-49: on Deny, return a clear error carrying the seat identity +
        // reason. No new error variant is needed — InvalidConfig covers
        // "admission refused" with a descriptive message (the orchestrator
        // has no dedicated "admission refused" arm, and the task forbids
        // inventing one). On Queued, fall through to J-48 with
        // seat_blocked forcing skip_local (same effect as AdmitQueued:
        // skip local backends, try cloud chain if any).
        if let SeatAdmitDecision::Deny { reason } = &seat_decision {
            let msg = format!(
                "seat admission denied for seat '{}': {reason}",
                self.seat_identity
            );
            // Release the inflight slot that admit did NOT increment (Deny
            // never increments inflight), so the registry stays balanced.
            // (release is a no-op when inflight is 0 — safe.)
            self.seat_registry.write().await.release(&self.seat_identity);
            return Err(RuntimeOrchestratorError::InvalidConfig { msg });
        }

        // J-48: local serving tier router pre-step. Before the fallback
        // chain, ask the LocalServingRouter whether to serve locally,
        // queue, hibernate+serve, or spill to cloud. The decision is
        // binding: SpillToCloud/AdmitQueued/Wait skip local backends in
        // the chain; ServeLocal/HibernateThenServe proceed and bump
        // tier_residency inflight so the map is not inert.
        let serving_decision = {
            let residency_guard = self.tier_residency.read().await;
            let cloud_cfg = CloudConfig::default();
            let cloud_available = !cloud_cfg.air_gapped;
            let router = LocalServingRouter::new();
            let requested_tier = VramTier::from_env();
            let input = DecideInput {
                requested_tier,
                residency: &residency_guard,
                cloud_available,
                spend: None,
            };
            router.decide(&input)
        };

        // J-48: write tier_residency so the map has writers, not just
        // readers. ServeLocal bumps inflight + marks warm;
        // HibernateThenServe evicts the small tier then warms big;
        // SpillToCloud/AdmitQueued/Wait leave residency unchanged (no
        // local serve this turn). The inflight is decremented after the
        // chain walk via release_tier_inflight.
        let served_tier: Option<VramTier> = match &serving_decision {
            LocalServingDecision::ServeLocal { tier } => Some(*tier),
            LocalServingDecision::HibernateThenServe { evict, serve } => {
                let mut guard = self.tier_residency.write().await;
                guard.insert(*evict, TierState::cold());
                Some(*serve)
            }
            _ => None,
        };
        if let Some(tier) = served_tier {
            let mut guard = self.tier_residency.write().await;
            let entry = guard.entry(tier).or_insert_with(TierState::cold);
            entry.warm = true;
            entry.inflight = entry.inflight.saturating_add(1);
            if tier == VramTier::High {
                for (t, s) in guard.iter_mut() {
                    if *t != VramTier::High {
                        s.warm = false;
                        s.inflight = 0;
                    }
                }
            }
        }

        // Emit the decision audit row via E-31 RouteDecisionEvent
        // (prefer-reuse, net 0 SSB). Wire-dark unless
        // ELAI_ROUTE_DECISION_SSB_ENABLE=1.
        if crate::cloud_mode::route_decision_ssb_enabled() {
            if let Some(ref ssb) = self.ssb {
                let kind = match &serving_decision {
                    LocalServingDecision::ServeLocal { .. }
                    | LocalServingDecision::AdmitQueued { .. }
                    | LocalServingDecision::Wait => {
                        crate::ssb::RouteDecisionEventKind::LocalRouteKept
                    }
                    LocalServingDecision::HibernateThenServe { .. } => {
                        crate::ssb::RouteDecisionEventKind::TierModelSwapped
                    }
                    LocalServingDecision::SpillToCloud { .. } => {
                        crate::ssb::RouteDecisionEventKind::CloudRouteChosen
                    }
                };
                let payload = crate::ssb::RouteDecisionEventPayload {
                    kind,
                    route: serving_decision.as_str().to_string(),
                    score: 0.0,
                    ms: 0,
                    reason: match &serving_decision {
                        LocalServingDecision::SpillToCloud { reason } => reason.clone(),
                        _ => String::new(),
                    },
                    session_id: Some(ssb.session_id().to_owned()),
                    details_json: {
                        // J-49: when the seat pre-filter did not Admit,
                        // augment the RouteDecisionEvent details_json with
                        // seat_id + seat_admit + reason so the audit row
                        // carries the seat dimension (prefer-reuse, net 0
                        // SSB — no new variant).
                        let mut base = serving_decision.details_json();
                        if let Some(reason) = &seat_block_reason {
                            let seat_admit = if seat_admitted {
                                "admit"
                            } else if matches!(seat_decision, SeatAdmitDecision::Queued { .. }) {
                                "queued"
                            } else {
                                "deny"
                            };
                            let seat_obj = serde_json::json!({
                                "seat_id": self.seat_identity,
                                "seat_admit": seat_admit,
                                "reason": reason,
                                "known_seats": self.seat_registry.read().await.seat_ids(),
                            });
                            // Merge seat_obj into the base object.
                            if let serde_json::Value::Object(ref mut map) = base {
                                if let serde_json::Value::Object(seat_map) = seat_obj {
                                    for (k, v) in seat_map {
                                        map.insert(k, v);
                                    }
                                }
                            }
                            base
                        } else {
                            base
                        }
                    },
                    ts_ms: now_ms(),
                };
                if let Err(e) = ssb
                    .write_payload(
                        "runtime_orchestrator",
                        0,
                        SsbPayload::RouteDecisionEvent(payload),
                        Vec::new(),
                    )
                    .await
                {
                    warn!(
                        target = "runtime_orchestrator",
                        error = %e,
                        "ssb write_payload failed for local_serving route decision"
                    );
                }
            }
        }

        // J-48: apply the decision to the fallback chain. SpillToCloud,
        // AdmitQueued, and Wait skip local backends so the decision is
        // not discarded. ServeLocal and HibernateThenServe proceed with
        // the full chain (residency was already bumped above).
        // J-49: a Queued seat also forces skip_local (same effect as
        // AdmitQueued — skip local backends, try cloud chain if any).
        let skip_local = matches!(
            &serving_decision,
            LocalServingDecision::SpillToCloud { .. }
                | LocalServingDecision::AdmitQueued { .. }
                | LocalServingDecision::Wait
        ) || !seat_admitted;
        let chain: Vec<RuntimeBackend> = if skip_local {
            if let LocalServingDecision::SpillToCloud { ref reason } = serving_decision {
                info!(
                    target = "runtime_orchestrator",
                    role = ?role,
                    reason = %reason,
                    "J-48 local serving router spilled to cloud"
                );
            }
            chain.into_iter().filter(|b| !b.is_local()).collect()
        } else {
            chain
        };

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
        // J-28 T6: joined input text for the client-side estimator seam
        // (fires only when a backend reports (0, 0) usage). Computed once
        // -- `messages` is not mutated across fallback-chain iterations.
        let estimator_input_text: String = messages
            .iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        for backend in chain {
            tried.push(backend);
            let adapter = match self.ensure_adapter(backend).await {
                Ok(a) => {
                    // J-30 T7a: stamp the turn-scoped pin on the first
                    // successful ensure_adapter in the chain. turn_id == 0
                    // (pre-warm sentinel) never stamps.
                    if turn_id != 0 {
                        let mut lock_guard = self.turn_lock.write().await;
                        match *lock_guard {
                            Some(pin) if pin.turn_id == turn_id => {}
                            _ => {
                                *lock_guard = Some(TurnBackendPin {
                                    backend,
                                    turn_id,
                                });
                            }
                        }
                    }
                    a
                }
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

            // J-28 RECON_NEEDED-4: CompletionResponse carries no elapsed_ms;
            // measure it with a seam-local wall-clock instead.
            let call_started = Instant::now(); // entropy-allow: monotonic start-marker for live backend-call latency telemetry only; a seeded clock would not measure real provider duration
            // J-54 UNIT7: free-pool on a verification role stamps trust
            // downgrade (non_reproducible) before the complete call.
            if backend == RuntimeBackend::FreePool {
                let _marker = apply_trust_marker(&agent_role_to_snake(role));
            }
            match adapter.complete(messages.clone(), config.clone()).await {
                Ok(response) => {
                    let call_elapsed_ms =
                        u64::try_from(call_started.elapsed().as_millis()).unwrap_or(u64::MAX);
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

                        // J-28 T3/T6: the uniform UsageTurnEvent row, beside
                        // BackendCostAccrued (never replacing it --
                        // BackendCostAccruedPayload carries no model/task/
                        // session key). RISK-1: unlike the local-arm's
                        // parse-error suppression below, this row is NEVER
                        // suppressed on (0,0) -- the estimator seam fires
                        // instead (Kind=EstimateApplied), so a zero-usage
                        // turn never vanishes silently.
                        let (usage_tokens_in, usage_tokens_out, usage_kind) =
                            crate::usage_ledger::resolve_turn_usage(
                                tokens_in,
                                tokens_out,
                                &estimator_input_text,
                                &response.text,
                            );
                        let usage_est_cost_micros = match usage_kind {
                            UsageTurnEventKind::TurnRecorded => est_cost_micros,
                            // Estimated tokens never carry a priced dollar
                            // figure (an estimate-derived cost would look
                            // measured but is not); the rollup layer also
                            // treats EstimateApplied cost as Option::None.
                            UsageTurnEventKind::EstimateApplied
                            | UsageTurnEventKind::NormalizationApplied => 0,
                        };
                        // T2: assemble the normalized sample AFTER T6's
                        // estimator resolution (ROUTE B, client-side,
                        // keyed by the lane tag).
                        let normalized = crate::usage_ledger::NormalizedUsage::local(
                            elai_core::TokenUsage {
                                input_tokens: usage_tokens_in,
                                output_tokens: usage_tokens_out,
                                cache_creation_input_tokens: 0,
                                cache_read_input_tokens: 0,
                            },
                        );
                        let usage_payload =
                            SsbPayload::UsageTurnEvent(UsageTurnEventPayload {
                                identity: "local".to_string(),
                                lane: normalized.lane,
                                backend: backend.canonical_name().to_string(),
                                model: response.model.clone(),
                                // GAP RISK-3: task_id is not threaded into
                                // try_complete's signature; None is honest,
                                // never a stand-in for a real id.
                                task_id: None,
                                session_id: Some(ssb.session_id().to_owned()),
                                input_tokens: usage_tokens_in,
                                output_tokens: usage_tokens_out,
                                cache_creation_tokens: 0,
                                cache_read_tokens: 0,
                                est_cost_micros: usage_est_cost_micros,
                                elapsed_ms: call_elapsed_ms,
                                semantics_tag: normalized.semantics_tag,
                                kind: usage_kind,
                                ts_ms: now_ms(),
                            });
                        if let Err(e) = ssb
                            .write_payload("runtime_orchestrator", 0, usage_payload, Vec::new())
                            .await
                        {
                            warn!(
                                target = "runtime_orchestrator",
                                error = %e,
                                "ssb write_payload failed for usage_turn_event (paid)"
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

                        // J-28 T3/T6 -- RISK-1: the UsageTurnEvent row is
                        // deliberately NOT gated on tokens_in>0||tokens_out>0
                        // (that gate above exists ONLY to suppress
                        // BackendCostAccrued parse-error noise). A (0,0)
                        // local turn here fires the estimator seam
                        // (Kind=EstimateApplied) instead of vanishing.
                        let (usage_tokens_in, usage_tokens_out, usage_kind) =
                            crate::usage_ledger::resolve_turn_usage(
                                tokens_in,
                                tokens_out,
                                &estimator_input_text,
                                &response.text,
                            );
                        // J-28 FIX: the local-lane NormalizedUsage::local +
                        // est_cost_micros: 0 payload is correct ONLY for
                        // genuinely local backends. A non-local backend
                        // without a cost_model_for entry (e.g. AnthropicApi,
                        // Sglang, Bedrock) must NOT be persisted as
                        // lane=Local with a zero cost -- route it through
                        // the cloud lane with an honest est_cost_micros: 0
                        // (unknown, mirroring grounding_client.rs's unpriced
                        // snapshot arm at line 998).
                        let normalized = if backend.is_local() {
                            crate::usage_ledger::NormalizedUsage::local(
                                elai_core::TokenUsage {
                                    input_tokens: usage_tokens_in,
                                    output_tokens: usage_tokens_out,
                                    cache_creation_input_tokens: response.cache_creation_input_tokens,
                                    cache_read_input_tokens: response.cache_read_input_tokens,
                                },
                            )
                        } else {
                            crate::usage_ledger::NormalizedUsage::cloud(
                                elai_core::TokenUsage {
                                    input_tokens: usage_tokens_in,
                                    output_tokens: usage_tokens_out,
                                    cache_creation_input_tokens: 0,
                                    cache_read_input_tokens: 0,
                                },
                            )
                        };
                        let usage_payload =
                            SsbPayload::UsageTurnEvent(UsageTurnEventPayload {
                                identity: "local".to_string(),
                                lane: normalized.lane,
                                backend: backend.canonical_name().to_string(),
                                model: response.model.clone(),
                                task_id: None,
                                session_id: Some(ssb.session_id().to_owned()),
                                input_tokens: usage_tokens_in,
                                output_tokens: usage_tokens_out,
                                cache_creation_tokens: response.cache_creation_input_tokens,
                                cache_read_tokens: response.cache_read_input_tokens,
                                // Local backends have zero marginal API cost
                                // (mirrors the BackendCostAccrued
                                // est_cost_micros: 0 arm above). Non-local
                                // backends without a cost_model_for entry
                                // also land here with 0 -- an honest unknown
                                // (no pricing snapshot available at this
                                // seam, unlike grounding_client.rs which
                                // computes from snapshot.priced), never a
                                // silently-wrong sonnet-derived figure.
                                est_cost_micros: 0,
                                elapsed_ms: call_elapsed_ms,
                                semantics_tag: normalized.semantics_tag,
                                kind: usage_kind,
                                ts_ms: now_ms(),
                            });
                        if let Err(e) = ssb
                            .write_payload("runtime_orchestrator", 0, usage_payload, Vec::new())
                            .await
                        {
                            warn!(
                                target = "runtime_orchestrator",
                                error = %e,
                                "ssb write_payload failed for usage_turn_event (local)"
                            );
                        }
                    }
                    if let Some(ref ssb) = self.ssb {
                        if backend.is_local() {
                            let read = response.cache_read_input_tokens as u64;
                            let create = response.cache_creation_input_tokens as u64;
                            let (kind, reason) = if read > 0 && create > 0 {
                                (
                                    crate::ssb::LocalPrefixCacheKind::TailReprefill,
                                    "tail_refilled",
                                )
                            } else if read > 0 {
                                (
                                    crate::ssb::LocalPrefixCacheKind::HeadCacheHit,
                                    "head_cache_hit",
                                )
                            } else {
                                (
                                    crate::ssb::LocalPrefixCacheKind::HeadCacheMiss,
                                    "no_cache_read",
                                )
                            };
                            crate::prefix_cache_ssb::emit_local_prefix_cache_event(
                                ssb, kind, read, create, reason,
                            );
                        }
                    }
                    // I-34: run the CRANE grammar gate on successful completion.
                    // Non-fatal: gate errors are warn-logged; the response is
                    // always returned to the caller (enforcement routing is a
                    // forward dep, see WIRE-DARK[I-34-enforce] in run_grammar_gate_if_eligible).
                    Self::run_grammar_gate_if_eligible(
                        &self.ssb,
                        role,
                        &config,
                        grammar_manifest_path.as_ref(),
                        &response.text,
                    ).await;
                    if let Some(tier) = served_tier {
                        self.release_tier_inflight(tier).await;
                    }
                    if seat_admitted {
                        self.release_seat().await;
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
                        if let Some(tier) = served_tier {
                            self.release_tier_inflight(tier).await;
                        }
                        if seat_admitted {
                            self.release_seat().await;
                        }
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
        if let Some(tier) = served_tier {
            self.release_tier_inflight(tier).await;
        }
        if seat_admitted {
            self.release_seat().await;
        }
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
        // J-30 T7a: turn_id=0 sentinel — pre-warm is a prediction, not a
        // sequence commitment; the turn-scoped lock is never stamped or
        // checked for pre-warm.
        let adapter = match self.for_role(next_role, 0).await {
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

    /// I-34: run the CRANE grammar gate for a single completed response if
    /// the completion is grammar-eligible and `config.grammar_gate_schema_json`
    /// is set. Non-fatal: any gate error (SSB write, manifest I/O) is logged
    /// at `warn` and the function returns without blocking the response.
    ///
    /// Eligibility: fires ONLY when
    ///   (a) `config.decoding_config` contains a `Grammar { crane_adaptive: true }` mode, AND
    ///   (b) `config.grammar_gate_schema_json` is `Some`.
    ///
    /// When schema_json is `None`, warns once and returns. When `ssb` is `None`,
    /// the gate cannot emit SSB rows and is skipped silently (no SSB = no gate).
    ///
    /// Manifest path resolution: uses `config_manifest_path` when `Some`,
    /// otherwise derives the conventional path from `current_dir()`.
    async fn run_grammar_gate_if_eligible(
        ssb: &Option<Arc<Ssb>>,
        role: AgentRole,
        config: &CompletionConfig,
        config_manifest_path: Option<&PathBuf>,
        decoded_output: &str,
    ) {
        // Gate needs an SSB handle to emit GrammarGateEvent rows.
        let ssb = match ssb.as_ref() {
            Some(s) => s,
            None => return,
        };

        // Gate needs a crane-adaptive Grammar decoding mode.
        let decoding_mode = match config.decoding_config.as_ref() {
            Some(dc) => &dc.mode,
            None => return,
        };
        if !matches!(decoding_mode, DecodingMode::Grammar { crane_adaptive: true, .. }) {
            return;
        }

        // Gate needs a schema_json to re-validate the output against.
        let schema_json = match config.grammar_gate_schema_json.as_deref() {
            Some(s) => s,
            None => {
                warn!(
                    target = "runtime_orchestrator",
                    ?role,
                    "I-34 grammar gate skipped: no schema_json on CompletionConfig"
                );
                return;
            }
        };

        // Resolve manifest path: prefer RuntimeConfig field, fall back to cwd + convention.
        let manifest_path: PathBuf = match config_manifest_path {
            Some(p) => p.clone(),
            None => match std::env::current_dir() {
                Ok(cwd) => default_manifest_path(&cwd),
                Err(e) => {
                    warn!(
                        target = "runtime_orchestrator",
                        error = %e,
                        "I-34 grammar gate: current_dir failed, skipping gate"
                    );
                    return;
                }
            },
        };

        let session_id = format!("runtime_orchestrator::{role:?}");

        // WIRE-DARK[I-34-enforce]: enforcement routing (PostParseRejected / PinViolation
        // returning Err on try_complete and routing to dispatch_conformal_refusal) is a
        // forward dep. This changeset: observe + emit only; return Ok normally on any
        // gate result. See wire-dark-allowlist.toml row "I-34-enforce".
        match call_grammar_gate(ssb, &session_id, decoding_mode, &manifest_path, decoded_output, schema_json).await {
            Ok(result) => {
                info!(
                    target = "runtime_orchestrator",
                    ?role,
                    result = ?result,
                    "I-34 grammar gate completed"
                );
            }
            Err(e) => {
                warn!(
                    target = "runtime_orchestrator",
                    ?role,
                    error = %e,
                    "I-34 grammar gate error (non-fatal, completion proceeds)"
                );
            }
        }
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
            .for_role(AgentRole::Executor, 0)
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
            .for_role(AgentRole::Executor, 0)
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
                0,
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
            .for_role(AgentRole::Executor, 0)
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
            .try_complete(AgentRole::Executor, messages, CompletionConfig::default(), 0)
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

    // -----------------------------------------------------------------
    // J-30 T7a — turn-scoped backend lock unit tests.
    //
    // These tests exercise the turn-scoped pin: once a backend is
    // resolved for a turn_id != 0, a subsequent call with the same
    // turn_id that would resolve a DIFFERENT backend returns
    // BackendLockedMidSequence. turn_id == 0 (pre-warm sentinel) never
    // stamps or checks the lock.

    #[tokio::test]
    async fn mid_sequence_swap_refused() {
        // Step 1: config with Executor -> Ollama, fallback [Ollama, Sidecar].
        let mut cfg = cfg_with_primary(RuntimeBackend::Ollama);
        cfg.fallback_chain = vec![RuntimeBackend::Ollama, RuntimeBackend::Sidecar];
        let orch = RuntimeOrchestrator::new(cfg);

        // Step 2: resolve for turn_id=1 — stamps the lock on Ollama.
        let adapter = orch
            .for_role(AgentRole::Executor, 1)
            .await
            .expect("must resolve Ollama");
        assert_eq!(adapter.backend(), RuntimeBackend::Ollama);

        // Step 3: swap config so Executor now maps to Sidecar (different
        // backend). set_config clears the adapter cache but does NOT
        // clear the turn_lock (the lock is turn-scoped, not config-scoped).
        let mut new_cfg = cfg_with_primary(RuntimeBackend::Sidecar);
        new_cfg.fallback_chain = vec![RuntimeBackend::Sidecar, RuntimeBackend::Ollama];
        orch.set_config(new_cfg).await;

        // Step 4: same turn_id=1, primary is now Sidecar. The lock holds
        // Ollama, so the swap must be refused.
        let result = orch.for_role(AgentRole::Executor, 1).await;
        match result {
            Err(RuntimeOrchestratorError::BackendLockedMidSequence {
                locked,
                attempted,
                turn_id,
            }) => {
                assert_eq!(locked, RuntimeBackend::Ollama);
                assert_eq!(attempted, RuntimeBackend::Sidecar);
                assert_eq!(turn_id, 1);
            }
            Ok(_) => panic!("expected BackendLockedMidSequence, got Ok(..)"),
            Err(e) => panic!("expected BackendLockedMidSequence, got Err: {e}"),
        }
    }

    #[tokio::test]
    async fn release_turn_lock_allows_next_turn() {
        let mut cfg = cfg_with_primary(RuntimeBackend::Ollama);
        cfg.fallback_chain = vec![RuntimeBackend::Ollama, RuntimeBackend::Sidecar];
        let orch = RuntimeOrchestrator::new(cfg);

        // Stamp the lock on Ollama for turn_id=1.
        let adapter = orch
            .for_role(AgentRole::Executor, 1)
            .await
            .expect("must resolve Ollama");
        assert_eq!(adapter.backend(), RuntimeBackend::Ollama);

        // Release the lock for turn_id=1.
        orch.release_turn_lock(1).await;

        // Swap config to Sidecar.
        let mut new_cfg = cfg_with_primary(RuntimeBackend::Sidecar);
        new_cfg.fallback_chain = vec![RuntimeBackend::Sidecar, RuntimeBackend::Ollama];
        orch.set_config(new_cfg).await;

        // turn_id=2 should resolve Sidecar freely (lock released, new turn).
        let adapter = orch
            .for_role(AgentRole::Executor, 2)
            .await
            .expect("must resolve Sidecar after lock release");
        assert_eq!(adapter.backend(), RuntimeBackend::Sidecar);
    }

    #[tokio::test]
    async fn prewarm_does_not_stamp_lock() {
        let orch = RuntimeOrchestrator::new(cfg_with_primary(RuntimeBackend::Ollama));

        // turn_id=0 is the pre-warm sentinel — the lock must NOT be stamped.
        let _adapter = orch
            .for_role(AgentRole::Executor, 0)
            .await
            .expect("must resolve Ollama");

        // Verify the lock field is still None.
        let lock_guard = orch.turn_lock.read().await;
        assert!(
            lock_guard.is_none(),
            "pre-warm (turn_id=0) must not stamp the turn lock"
        );
    }
}
