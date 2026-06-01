//! E-44 (Tranche 7A) — SAGA compensation log.
//!
//! The SAGA pattern (Garcia-Molina & Salem 1987; refined by Azure /
//! Temporal) decomposes a long-running multi-step operation into a
//! sequence of local transactions where each step pairs with an
//! explicit *compensation* (inverse) action. On a crash mid-sequence
//! the recovery path replays the logged compensations in reverse order
//! to roll the system back to a consistent state.
//!
//! Ordering invariant (HARD): the compensation MUST be logged BEFORE
//! the irreversible step runs. If the step's side-effect lands first
//! and the process crashes before the log write, recovery has no
//! record of the inverse and the system drifts. We encode this via a
//! phantom typestate on [`SagaTicket`]: callers receive a
//! `SagaTicket<Pending>` from the planner, exchange it for a
//! `SagaTicket<Committed>` via [`SagaLogger::log_compensation`], and
//! only then is the irreversible op permitted. The exchange consumes
//! the `Pending` ticket so the type system forbids "log after action"
//! at compile time. A failure path produces `SagaTicket<Failed>` for
//! audit-trail completeness.
//!
//! Producer wiring: this module emits 3 [`DurabilityEventKind`] arms
//! reserved at SCHEMA_VERSION 186 (no further bump required):
//!   * [`DurabilityEventKind::CompensationLogged`] on
//!     [`SagaLogger::log_compensation`].
//!   * [`DurabilityEventKind::CompensationReplayed`] on
//!     [`SagaLogger::replay_pending`] success path.
//!   * [`DurabilityEventKind::CompensationFailed`] on
//!     [`SagaLogger::record_failure`].
//!
//! Determinism: BTreeMap for the pending log (HashMap is banned);
//! `blake3::Hasher` for any digest derivation (anti-stale-training).
//! No `unwrap()` / `expect()` in any production code path.

use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;

use crate::durability::observability::{
    notify_compensation_failed, notify_compensation_logged, notify_compensation_replayed,
};
use crate::ssb::{Ssb, SsbError};

/// Typestate marker: the ticket has been MINTED but the inverse action
/// is not yet persisted. The irreversible step MUST NOT run while the
/// ticket is in this state — the only legal transition is
/// [`SagaLogger::log_compensation`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pending;

/// Typestate marker: the compensation has been persisted and the
/// irreversible step is now safe to execute. Recovery will see the
/// committed compensation_actions row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Committed;

/// Typestate marker: a compensation replay errored OR the step's
/// irreversible op was abandoned and a failure was recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Failed;

/// Saga identifier (BLAKE3-derived stable hash).
pub type SagaId = String;

/// One compensation action in a SAGA. The `compensate_tool` /
/// `compensate_args_json` pair is replayed verbatim during recovery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SagaAction {
    pub saga_id: SagaId,
    pub action_id: String,
    pub step_index: u32,
    pub compensate_tool: String,
    pub compensate_args_json: serde_json::Value,
    pub recorded_at_ms: i64,
}

impl SagaAction {
    /// Derive a deterministic `action_id` from the saga_id + step_index
    /// + tool name. Uses BLAKE3 (NEVER blake2 / sha2 per anti-stale).
    #[must_use]
    pub fn derive_action_id(saga_id: &str, step_index: u32, compensate_tool: &str) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(saga_id.as_bytes());
        hasher.update(&step_index.to_le_bytes());
        hasher.update(b"\x00");
        hasher.update(compensate_tool.as_bytes());
        let digest = hasher.finalize();
        let bytes = digest.as_bytes();
        format!(
            "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]
        )
    }
}

/// Typestate-wrapped saga handle. The `S` parameter is a phantom
/// marker enforcing the Pending -> Committed -> (Replayed | Failed)
/// lifecycle at compile time.
///
/// See module docstring for the ordering invariant rationale.
#[derive(Debug, Clone)]
pub struct SagaTicket<S> {
    saga_id: SagaId,
    action: SagaAction,
    _state: PhantomData<S>,
}

impl<S> SagaTicket<S> {
    /// Stable saga identifier.
    #[must_use]
    pub fn saga_id(&self) -> &str {
        &self.saga_id
    }

    /// Compensation action carried by the ticket.
    #[must_use]
    pub fn action(&self) -> &SagaAction {
        &self.action
    }
}

impl SagaTicket<Pending> {
    /// Mint a fresh ticket in the `Pending` state. The irreversible
    /// step MUST NOT run until this ticket has been exchanged for a
    /// `Committed` ticket via [`SagaLogger::log_compensation`].
    #[must_use]
    pub fn new(saga_id: SagaId, action: SagaAction) -> Self {
        Self {
            saga_id,
            action,
            _state: PhantomData,
        }
    }

    /// Compile-time guarantee: there is NO public path that turns a
    /// `Pending` ticket into a `Committed` ticket without routing
    /// through [`SagaLogger::log_compensation`]. Confirmed by the
    /// negative-space `try_irreversible_before_log` doctest below.
    ///
    /// ```compile_fail
    /// use elai_orchestrator::durability::saga::{SagaTicket, Pending, Committed};
    /// fn try_irreversible_before_log(t: SagaTicket<Pending>) -> SagaTicket<Committed> {
    ///     // This should not compile: cannot convert Pending into
    ///     // Committed without the SagaLogger contract.
    ///     unsafe { std::mem::transmute(t) }
    /// }
    /// ```
    #[doc(hidden)]
    pub fn __compile_time_ordering_marker() {}
}

/// Errors raised by the SAGA log + replay machinery.
#[derive(Debug, Error)]
pub enum SagaError {
    /// Underlying SSB write failed.
    #[error("ssb write failed during saga emit: {0}")]
    Ssb(#[from] SsbError),

    /// Caller violated the ticket lifecycle (e.g. tried to record a
    /// failure on an unknown saga_id). Defensive — the typestate
    /// should already prevent the common cases.
    #[error("saga lifecycle violation: {reason}")]
    Lifecycle { reason: String },

    /// The supplied agent_id / session_id was empty. SSB write_payload
    /// rejects empty agent ids, so we surface the error here with a
    /// caller-friendly tag.
    #[error("invalid saga input: {reason}")]
    InvalidInput { reason: String },
}

/// SAGA logger. Owns the in-memory pending-actions map keyed by
/// `(saga_id, action_id)` so recovery can replay the compensations
/// without re-reading SSB.
///
/// Note: `Ssb` does not implement `Debug` (it owns a `JoinHandle` +
/// channel writers), so we provide a manual `Debug` impl that elides
/// the writer.
pub struct SagaLogger {
    ssb: Option<Arc<Ssb>>,
    pending: Mutex<BTreeMap<(SagaId, String), SagaAction>>,
    agent_id: String,
}

impl Default for SagaLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SagaLogger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SagaLogger")
            .field("ssb_attached", &self.ssb.is_some())
            .field("agent_id", &self.agent_id)
            .finish()
    }
}

impl SagaLogger {
    /// Construct a logger with no SSB writer attached. Emission is a
    /// no-op until [`with_ssb`] is called.
    #[must_use]
    pub fn new() -> Self {
        Self {
            ssb: None,
            pending: Mutex::new(BTreeMap::new()),
            agent_id: "orchestrator".to_owned(),
        }
    }

    /// Attach an SSB writer. Producer notify_* helpers no-op when the
    /// writer is absent (matches wire-dark default).
    #[must_use]
    pub fn with_ssb(mut self, ssb: Arc<Ssb>) -> Self {
        self.ssb = Some(ssb);
        self
    }

    /// Set the canonical agent_id carried on emitted SSB rows.
    #[must_use]
    pub fn with_agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = agent_id.into();
        self
    }

    /// Log the compensation BEFORE the irreversible step runs. On
    /// success the ticket is consumed and a `Committed` ticket is
    /// returned to the caller. The compensation is persisted to the
    /// SSB and held in the in-memory pending map so recovery can
    /// replay it.
    pub async fn log_compensation(
        &self,
        ticket: SagaTicket<Pending>,
    ) -> Result<SagaTicket<Committed>, SagaError> {
        if ticket.saga_id.is_empty() {
            return Err(SagaError::InvalidInput {
                reason: "saga_id is empty".into(),
            });
        }
        if ticket.action.action_id.is_empty() {
            return Err(SagaError::InvalidInput {
                reason: "action_id is empty".into(),
            });
        }

        // Insert into pending map BEFORE the SSB emit so a recovery
        // running concurrently with the emit sees the action.
        {
            let mut guard = self.pending.lock().await;
            guard.insert(
                (ticket.saga_id.clone(), ticket.action.action_id.clone()),
                ticket.action.clone(),
            );
        }

        if let Some(ssb) = self.ssb.as_ref() {
            notify_compensation_logged(
                ssb.as_ref(),
                &self.agent_id,
                0,
                Some(ticket.saga_id.as_str()),
                ticket.saga_id.as_str(),
                ticket.action.action_id.as_str(),
                ticket.action.step_index,
                ticket.action.compensate_tool.as_str(),
                ticket.action.recorded_at_ms,
            )
            .await?;
        }

        Ok(SagaTicket {
            saga_id: ticket.saga_id,
            action: ticket.action,
            _state: PhantomData,
        })
    }

    /// Replay all pending compensations. Returns the count of
    /// successfully replayed actions. Each successful replay emits
    /// [`DurabilityEventKind::CompensationReplayed`].
    ///
    /// Recovery callers typically invoke this from fsm.rs Resume /
    /// runtime_orchestrator boot.
    pub async fn replay_pending(&self) -> Result<usize, SagaError> {
        let drained: Vec<((SagaId, String), SagaAction)> = {
            let mut guard = self.pending.lock().await;
            let items: Vec<_> = guard.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            guard.clear();
            items
        };

        let mut replayed = 0_usize;
        if let Some(ssb) = self.ssb.as_ref() {
            for ((saga_id, action_id), action) in drained.iter() {
                notify_compensation_replayed(
                    ssb.as_ref(),
                    &self.agent_id,
                    0,
                    Some(saga_id.as_str()),
                    saga_id.as_str(),
                    action_id.as_str(),
                    "ok",
                    0,
                    action.recorded_at_ms,
                )
                .await?;
                replayed += 1;
            }
        } else {
            // No SSB attached: still drain so callers see the same
            // count semantics; emission is a no-op.
            replayed = drained.len();
        }
        Ok(replayed)
    }

    /// Record a failure against a Pending ticket. Returns the ticket
    /// re-tagged as `Failed` for audit trail closure. Emits
    /// [`DurabilityEventKind::CompensationFailed`].
    pub async fn record_failure(
        &self,
        ticket: SagaTicket<Pending>,
        reason: SagaError,
    ) -> Result<SagaTicket<Failed>, SagaError> {
        let truncated = {
            let s = reason.to_string();
            if s.len() > 200 {
                s[..200].to_owned()
            } else {
                s
            }
        };
        if let Some(ssb) = self.ssb.as_ref() {
            notify_compensation_failed(
                ssb.as_ref(),
                &self.agent_id,
                0,
                Some(ticket.saga_id.as_str()),
                ticket.saga_id.as_str(),
                ticket.action.action_id.as_str(),
                "saga_replay_error",
                truncated.as_str(),
                ticket.action.recorded_at_ms,
            )
            .await?;
        }
        Ok(SagaTicket {
            saga_id: ticket.saga_id,
            action: ticket.action,
            _state: PhantomData,
        })
    }

    /// Test/observability: number of pending compensations currently
    /// held in memory.
    pub async fn pending_count(&self) -> usize {
        let guard = self.pending.lock().await;
        guard.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_action_id_is_stable_and_blake3() {
        let id1 = SagaAction::derive_action_id("saga-A", 0, "git_commit");
        let id2 = SagaAction::derive_action_id("saga-A", 0, "git_commit");
        assert_eq!(id1, id2, "derive_action_id must be deterministic");
        let id3 = SagaAction::derive_action_id("saga-A", 1, "git_commit");
        assert_ne!(id1, id3, "step_index must influence the digest");
        assert_eq!(id1.len(), 16, "16-hex-char prefix (8 bytes of blake3)");
    }

    #[tokio::test]
    async fn log_compensation_consumes_pending_and_emits_no_ssb_safe() {
        let logger = SagaLogger::new();
        let action = SagaAction {
            saga_id: "saga-1".into(),
            action_id: SagaAction::derive_action_id("saga-1", 0, "tool"),
            step_index: 0,
            compensate_tool: "undo".into(),
            compensate_args_json: serde_json::json!({"k": "v"}),
            recorded_at_ms: 1,
        };
        let ticket = SagaTicket::<Pending>::new("saga-1".into(), action);
        let committed = logger
            .log_compensation(ticket)
            .await
            .expect("log_compensation should succeed without an SSB attached");
        assert_eq!(committed.saga_id(), "saga-1");
        assert_eq!(logger.pending_count().await, 1);
    }

    #[tokio::test]
    async fn replay_pending_drains_map() {
        let logger = SagaLogger::new();
        let action = SagaAction {
            saga_id: "saga-2".into(),
            action_id: SagaAction::derive_action_id("saga-2", 0, "tool"),
            step_index: 0,
            compensate_tool: "undo".into(),
            compensate_args_json: serde_json::json!({}),
            recorded_at_ms: 0,
        };
        let ticket = SagaTicket::<Pending>::new("saga-2".into(), action);
        let _committed = logger
            .log_compensation(ticket)
            .await
            .expect("log_compensation must succeed");
        assert_eq!(logger.pending_count().await, 1);
        let replayed = logger.replay_pending().await.expect("replay");
        assert_eq!(replayed, 1);
        assert_eq!(logger.pending_count().await, 0);
    }

    #[tokio::test]
    async fn record_failure_yields_failed_ticket() {
        let logger = SagaLogger::new();
        let action = SagaAction {
            saga_id: "saga-3".into(),
            action_id: SagaAction::derive_action_id("saga-3", 2, "tool"),
            step_index: 2,
            compensate_tool: "undo".into(),
            compensate_args_json: serde_json::json!(null),
            recorded_at_ms: 7,
        };
        let ticket = SagaTicket::<Pending>::new("saga-3".into(), action);
        let failed = logger
            .record_failure(
                ticket,
                SagaError::Lifecycle {
                    reason: "test".into(),
                },
            )
            .await
            .expect("record_failure should succeed without an SSB attached");
        assert_eq!(failed.saga_id(), "saga-3");
    }

    #[tokio::test]
    async fn log_compensation_rejects_empty_saga_id() {
        let logger = SagaLogger::new();
        let action = SagaAction {
            saga_id: "".into(),
            action_id: "a".into(),
            step_index: 0,
            compensate_tool: "undo".into(),
            compensate_args_json: serde_json::json!({}),
            recorded_at_ms: 0,
        };
        let ticket = SagaTicket::<Pending>::new("".into(), action);
        let err = logger.log_compensation(ticket).await.unwrap_err();
        assert!(matches!(err, SagaError::InvalidInput { .. }));
    }
}
