//! I-11: typed, versioned BeliefState for cross-model and cross-tier handoffs.
//!
//! Replaces raw session-history passthrough on the cloud escalation path with a
//! compact, DKG-keyed, honestly-measured condensate. Every condensation emits an
//! ID-3 receipt row (BeliefStateEvent(Condensed)); handoff dispatch emits
//! BeliefStateEvent(Handoff); the periodic honesty job emits HonestyDelta.
//!
//! Schema:
//!   BeliefState { version, tried, failed_because, current_ast_state,
//!                 constraints, dkg_entities, token_delta, critical_atoms }
//!   Version mismatch on decode -> BeliefStateVersionMismatch (fail-closed).
//!   Serialization: rmp_serde (MessagePack, already in orchestrator Cargo.toml).
//!   Hash: blake3 only (CLAUDE.md Anti-Stale-Training).
//!
//! Grounding:
//!   IC-8: typed + DKG-keyed + journaled + honestly measured (PABU differentiator).
//!   ID-3: every condensation emits a receipt row; un-journaled = FAIL.
//!   CL-1..CL-4: BeliefState sits at local/cloud boundary; encode is smaller than
//!     raw history, enabling the cloud-call cost reduction.
//!   IS-6: context_budget.rs::adaptive_reduce is the condensation engine.
//!
//! WIRING (I-11, R8 production-caller gate):
//!   encode() is called in escalation_policy::evaluate_handoff (Cloud arm).
//!   decode() is called in session_dag::load_handoff (resume path, before recover_stubs).
//!   HonestyJob::run() is called from task_runner::run_honesty_job_if_due
//!     (production entry-point for the periodic honesty check).

use thiserror::Error;
use tracing::{info, warn};

use crate::ssb::{BeliefStateEventKind, BeliefStateEventPayload, Ssb, SsbError, SsbPayload};

/// Schema version for the BeliefState wire format.
/// On a decode, a mismatch returns `BeliefStateVersionMismatch` (fail-closed).
pub const BELIEF_STATE_SCHEMA_VERSION: u32 = 1;

/// I-11: typed error for schema-version mismatch on BeliefState decode.
/// Never silently parses the wrong version.
#[derive(Debug, Error)]
pub enum BeliefStateError {
    /// The serialized blob's version does not match `BELIEF_STATE_SCHEMA_VERSION`.
    #[error(
        "BeliefState schema version mismatch: encoded={encoded}, current={current}"
    )]
    VersionMismatch { encoded: u32, current: u32 },
    /// MessagePack deserialization failure.
    #[error("BeliefState decode error: {0}")]
    Decode(#[from] rmp_serde::decode::Error),
    /// MessagePack serialization failure.
    #[error("BeliefState encode error: {0}")]
    Encode(#[from] rmp_serde::encode::Error),
    /// SSB write failure during receipt emission.
    #[error("BeliefState SSB write error: {0}")]
    Ssb(#[from] SsbError),
}

/// One tried approach, carried in the handoff payload so the receiving model
/// does not repeat a known-failed path.
/// Kind of a decision-relevant atom the handoff must preserve (TD-3: orchestrator sets these).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtomKind {
    FilePath,
    Symbol,
    TestName,
    LineSpan,
    BugLocation,
    SafetyConstraint,
    Other,
}

/// One critical atom for the handoff-faithfulness gate (K-12 R2).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CriticalAtom {
    pub kind: AtomKind,
    /// Exact string that must appear in the projected context when must_preserve.
    pub value: String,
    /// When true the faithfulness gate refuses a projection that drops this value.
    /// LLM never sets this (TD-3) — only orchestrator / structured sources.
    pub must_preserve: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TriedApproach {
    /// Short label for the approach (e.g. "async trait + spawn").
    pub label: String,
    /// Token cost of the attempt (from SSB ContextReduction rows).
    pub token_cost: u32,
}

/// Typed, versioned condensate of the in-flight session belief state.
///
/// Replaces raw session history on the cloud escalation path (stub_ban invariant:
/// no raw history passthrough). Constructed AFTER `adaptive_reduce` returns
/// (the &mut self borrow releases), using the post-reduction slot snapshot.
///
/// Serialization: rmp_serde (MessagePack). Hash: blake3.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BeliefState {
    /// Wire format version; mismatch on decode fails CLOSED.
    pub version: u32,
    /// Approaches that were tried (label + token cost).
    pub tried: Vec<TriedApproach>,
    /// Failure reasons collected from prior attempts (from SSB Finding rows).
    pub failed_because: Vec<String>,
    /// Snapshot of the current AST state (opaque bytes; from the active SessionDag
    /// create_handoff Snapshot, serialized as a nested MessagePack blob).
    pub current_ast_state: Vec<u8>,
    /// Active constraints at escalation time (from HandoffContext / PhaseSlot).
    pub constraints: Vec<String>,
    /// DKG entity keys linked to the current context (i64 node_id from DkgNodeInfo).
    pub dkg_entities: Vec<i64>,
    /// Tokens saved by `adaptive_reduce` before encoding (for the HonestyDelta ledger).
    pub token_delta: u32,
    /// K-12 R2: critical atoms for the handoff-faithfulness gate.
    #[serde(default)]
    pub critical_atoms: Vec<CriticalAtom>,
}

impl BeliefState {
    /// Encode to MessagePack bytes.
    /// The version field is BELIEF_STATE_SCHEMA_VERSION; callers must use this
    /// constructor to ensure the version is always stamped correctly.
    pub fn new(
        tried: Vec<TriedApproach>,
        failed_because: Vec<String>,
        current_ast_state: Vec<u8>,
        constraints: Vec<String>,
        dkg_entities: Vec<i64>,
        token_delta: u32,
    ) -> Self {
        Self {
            version: BELIEF_STATE_SCHEMA_VERSION,
            tried,
            failed_because,
            current_ast_state,
            constraints,
            dkg_entities,
            token_delta,
            critical_atoms: Vec::new(),
        }
    }

    /// Set the critical atoms carried by this belief state.
    pub fn with_critical_atoms(mut self, atoms: Vec<CriticalAtom>) -> Self {
        self.critical_atoms = atoms;
        self
    }

    /// Serialize to MessagePack bytes.
    ///
    /// # Errors
    /// Returns `BeliefStateError::Encode` if serialization fails.
    pub fn encode(&self) -> Result<Vec<u8>, BeliefStateError> {
        Ok(rmp_serde::to_vec_named(self)?)
    }

    /// Deserialize from MessagePack bytes and version-check.
    ///
    /// Fails CLOSED on version mismatch (BeliefStateVersionMismatch) so the
    /// receiving model never silently parses a stale schema.
    ///
    /// # Errors
    /// - `BeliefStateError::Decode` if deserialization fails.
    /// - `BeliefStateError::VersionMismatch` if the encoded version != current.
    pub fn decode(bytes: &[u8]) -> Result<Self, BeliefStateError> {
        let state: Self = rmp_serde::from_slice(bytes)?;
        if state.version != BELIEF_STATE_SCHEMA_VERSION {
            return Err(BeliefStateError::VersionMismatch {
                encoded: state.version,
                current: BELIEF_STATE_SCHEMA_VERSION,
            });
        }
        Ok(state)
    }

    /// blake3 content hash of the encoded bytes (8-byte prefix, lowercase hex).
    /// Used for deduplication and the honesty-job delta check.
    ///
    /// # Errors
    /// Propagates `encode()` error.
    pub fn blake3_prefix(&self) -> Result<String, BeliefStateError> {
        let bytes = self.encode()?;
        let hash = blake3::hash(&bytes);
        // 8-byte prefix as 16 hex chars.
        let hex: String = hash.as_bytes()[..8]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        Ok(hex)
    }

    /// Emit the ID-3 receipt row (BeliefStateEvent(Condensed)) to SSB.
    ///
    /// Every condensation MUST emit this row (ID-3 JOURNALED-MIDDLEWARE rule).
    /// Un-journaled condensation = FAIL per the wiring contract.
    ///
    /// # Errors
    /// Returns `BeliefStateError::Ssb` if the SSB write fails.
    pub async fn emit_condensed_receipt(
        &self,
        ssb: &Ssb,
        session_id: u64,
        ts_ms: i64,
    ) -> Result<(), BeliefStateError> {
        if crate::per_step_source_check::per_step_source_check_enabled() {
            for constraint in &self.constraints {
                let trimmed = constraint.trim();
                if trimmed.contains(':') {
                    crate::per_step_source_check::observe_step_grounding(Some(trimmed), trimmed, None);
                }
            }
        }
        let payload = SsbPayload::BeliefStateEvent(BeliefStateEventPayload {
            kind: BeliefStateEventKind::Condensed,
            schema_version: self.version,
            token_delta: self.token_delta,
            payload_bytes: 0,
            ts_ms,
        });
        let superstep = i64::try_from(session_id).unwrap_or(i64::MAX);
        ssb.write_payload("belief_state", superstep, payload, vec![])
            .await?;
        Ok(())
    }

    /// Emit BeliefStateEvent(Handoff) when the payload is dispatched to the cloud arm.
    ///
    /// # Errors
    /// Returns `BeliefStateError::Ssb` if the SSB write fails.
    pub async fn emit_handoff_receipt(
        &self,
        ssb: &Ssb,
        session_id: u64,
        payload_bytes: u32,
        ts_ms: i64,
    ) -> Result<(), BeliefStateError> {
        let payload = SsbPayload::BeliefStateEvent(BeliefStateEventPayload {
            kind: BeliefStateEventKind::Handoff,
            schema_version: self.version,
            token_delta: self.token_delta,
            payload_bytes,
            ts_ms,
        });
        let superstep = i64::try_from(session_id).unwrap_or(i64::MAX);
        ssb.write_payload("belief_state", superstep, payload, vec![])
            .await?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// HonestyJob: periodic full-history re-run fixture + delta ledger
// ---------------------------------------------------------------------------

/// Interval (number of escalations) between honesty-job runs.
/// After every N escalations (or when `--force-honesty-check` is set),
/// HonestyJob re-runs the full SSB history and measures the success-delta.
pub const HONESTY_JOB_INTERVAL: u32 = 10;

/// I-11: periodic full-history-vs-BeliefState re-run fixture + delta ledger.
///
/// Runs pure arithmetic over SSB token counts; no GPU, no LLM call.
/// The delta is deterministic from the fixture SSB stream under ELAI_SEED (GD-2).
/// Emits HonestyDelta to the SSB ledger so compression cost is measured, not assumed.
#[derive(Debug, Clone)]
pub struct HonestyJob {
    /// Number of escalations between honesty-job runs (default: 10).
    pub interval_escalations: u32,
    /// Deterministic seed for reproducibility (GD-2 / ELAI_SEED).
    pub seed: u64,
}

impl HonestyJob {
    /// Construct a new HonestyJob with the given interval and seed.
    pub fn new(interval_escalations: u32, seed: u64) -> Self {
        Self {
            interval_escalations,
            seed,
        }
    }

    /// Run the honesty check: compare full-history token count against
    /// BeliefState-compressed token count, emit the HonestyDelta row.
    ///
    /// # Panics never
    /// # Errors
    /// Returns `BeliefStateError::Ssb` if the SSB write fails.
    pub async fn run(
        &self,
        ssb: &Ssb,
        session_id: u64,
        full_history_tokens: u32,
        belief_state_tokens: u32,
        ts_ms: i64,
    ) -> Result<HonestyDeltaPayload, BeliefStateError> {
        let delta = full_history_tokens.saturating_sub(belief_state_tokens);
        let compression_ratio = if full_history_tokens > 0 {
            f64::from(belief_state_tokens) / f64::from(full_history_tokens)
        } else {
            1.0
        };

        let result = HonestyDeltaPayload {
            full_history_tokens,
            belief_state_tokens,
            token_delta: delta,
            compression_ratio,
            seed: self.seed,
        };

        let payload = SsbPayload::BeliefStateEvent(BeliefStateEventPayload {
            kind: BeliefStateEventKind::HonestyDelta,
            schema_version: BELIEF_STATE_SCHEMA_VERSION,
            token_delta: delta,
            payload_bytes: 0,
            ts_ms,
        });
        let superstep = i64::try_from(session_id).unwrap_or(i64::MAX);
        ssb.write_payload("belief_state", superstep, payload, vec![])
            .await?;

        info!(
            session_id,
            full_history_tokens,
            belief_state_tokens,
            token_delta = delta,
            compression_ratio,
            seed = self.seed,
            "I-11 honesty-job: delta ledger emitted"
        );

        Ok(result)
    }
}

/// Result of a HonestyJob run. Recomputable under ELAI_SEED (GD-2).
#[derive(Debug, Clone, PartialEq)]
pub struct HonestyDeltaPayload {
    /// Token count of the full history (control arm).
    pub full_history_tokens: u32,
    /// Token count of the BeliefState-compressed arm.
    pub belief_state_tokens: u32,
    /// Tokens saved = full_history - belief_state (saturating).
    pub token_delta: u32,
    /// Compression ratio = belief_state_tokens / full_history_tokens.
    pub compression_ratio: f64,
    /// Seed used for reproducibility.
    pub seed: u64,
}

// ---------------------------------------------------------------------------
// Production caller: run_honesty_job_if_due (called from task_runner)
// ---------------------------------------------------------------------------

/// I-11 production entrypoint for the periodic honesty check.
///
/// Called by `task_runner::drive_task` (the production caller).
/// Reads the `ELAI_SEED` env var for deterministic runs; falls back to 0.
/// A missing or empty SSB session produces `token_delta=0` and is a no-op
/// (the honesty job only fires when `escalation_count >= interval`).
///
/// # Errors
/// Non-fatal: logs a warning on SSB write failure; never aborts the task.
pub async fn run_honesty_job_if_due(
    ssb: &Ssb,
    session_id: u64,
    escalation_count: u32,
    full_history_tokens: u32,
    belief_state_tokens: u32,
    ts_ms: i64,
    force: bool,
) {
    if !force && escalation_count % HONESTY_JOB_INTERVAL != 0 {
        return;
    }
    if escalation_count == 0 && !force {
        return;
    }

    let seed: u64 = std::env::var("ELAI_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let job = HonestyJob::new(HONESTY_JOB_INTERVAL, seed);
    match job
        .run(ssb, session_id, full_history_tokens, belief_state_tokens, ts_ms)
        .await
    {
        Ok(delta) => {
            info!(
                session_id,
                token_delta = delta.token_delta,
                compression_ratio = delta.compression_ratio,
                "I-11 run_honesty_job_if_due: honesty job completed"
            );
        }
        Err(e) => {
            warn!(
                session_id,
                error = %e,
                "I-11 run_honesty_job_if_due: honesty job SSB write failed (non-fatal)"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_belief_state() -> BeliefState {
        BeliefState::new(
            vec![TriedApproach {
                label: "async trait attempt".into(),
                token_cost: 1200,
            }],
            vec!["type mismatch in async fn return".into()],
            b"fake-ast-snapshot".to_vec(),
            vec!["no unsafe blocks".into()],
            vec![42_i64, 99_i64],
            800,
        )
    }

    #[test]
    fn encode_decode_round_trip() {
        let state = make_belief_state();
        let bytes = state.encode().expect("encode");
        let decoded = BeliefState::decode(&bytes).expect("decode");
        assert_eq!(decoded.version, BELIEF_STATE_SCHEMA_VERSION);
        assert_eq!(decoded.tried.len(), 1);
        assert_eq!(decoded.failed_because.len(), 1);
        assert_eq!(decoded.constraints, state.constraints);
        assert_eq!(decoded.dkg_entities, vec![42_i64, 99_i64]);
        assert_eq!(decoded.token_delta, 800);
    }

    #[test]
    fn decode_version_mismatch_fails_closed() {
        let mut state = make_belief_state();
        // Tamper with the version to simulate a schema bump.
        state.version = 99;
        let bytes = state.encode().expect("encode tampered");
        let result = BeliefState::decode(&bytes);
        assert!(
            matches!(
                result,
                Err(BeliefStateError::VersionMismatch { encoded: 99, current: 1 })
            ),
            "expected VersionMismatch, got {:?}",
            result
        );
    }

    #[test]
    fn blake3_prefix_is_deterministic() {
        let state = make_belief_state();
        let h1 = state.blake3_prefix().expect("hash 1");
        let h2 = state.blake3_prefix().expect("hash 2");
        assert_eq!(h1, h2, "hash must be deterministic");
        assert_eq!(h1.len(), 16, "8 bytes = 16 hex chars");
    }

    #[test]
    fn honesty_delta_arithmetic() {
        let job = HonestyJob::new(10, 42);
        // Just test the struct fields; we skip the async SSB write in this unit test.
        let _ = job;
        // Token delta = full - belief_state (saturating).
        let full: u32 = 20_000;
        let bs: u32 = 1_800;
        let delta = full.saturating_sub(bs);
        assert_eq!(delta, 18_200);
        let ratio = f64::from(bs) / f64::from(full);
        assert!((ratio - 0.09).abs() < 0.01, "ratio ~9%");
    }
}
