//! E-6 — TypedAgentEnvelope: typed payload for inter-agent messaging.
//!
//! ## Why
//!
//! ELAI's existing AgentConnection (orchestrator/src/ipc_gateway.rs:78)
//! accepts free-text payloads on the inbound side. Plan task 1 wraps
//! every inter-agent message in a typed envelope so the orchestrator
//! can refuse / count unstructured payloads. The CAMEL pattern (per
//! arXiv:2511.09710 Echoing) shows that without typed envelopes,
//! agents amplify each other's identity drift through unstructured
//! mirror replies — 5-70% rates depending on framework.
//!
//! ## Wire format
//!
//! Postcard-serialised. The same encoding as IpcMessageType (this
//! crate already exports `to_wire_bytes` / `from_wire_bytes` at the
//! IpcFrame layer). Adding a new wire tag is NOT a protocol bump:
//! envelopes are payload-level wrappers, not framing-level. The
//! envelope-level wire version is a struct field (`envelope_v: u8`)
//! independent of `IPC_PROTOCOL_VERSION`.
//!
//! ## Content hash
//!
//! `content_hash` is the blake3 digest of the canonicalised payload
//! bytes. blake3 chosen per CLAUDE.md anti-stale rule (blake3 over
//! blake2/sha2 for non-RFC uses). The hash is informational — it
//! lets the orchestrator dedupe rebroadcast envelopes without
//! re-parsing the payload.
//!
//! ## Grounding
//!
//! - arxiv:2511.09710 (T1) — Echoing. Structured response protocol
//!   reduces identity drift from 55-70% to <10% on GPT/Sonnet variants.
//! - arxiv:2503.13657 (T1) — MAST FM-2.5 IgnoredOtherAgentInput +
//!   FM-2.6 ReasoningActionMismatch + FM-1.4 LossOfConversationHistory.
//!   Typed envelopes are a structural prevention vector for FM-1.4 and
//!   FM-2.6 (sender_role + receiver_role + ts_ms make message origin
//!   explicit, prevent silent context-loss).

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::AgentRole;

/// Wire-version of the envelope schema. Bumped if the envelope struct
/// shape changes. Independent of [`crate::IPC_PROTOCOL_VERSION`] —
/// envelopes are payload-level, not frame-level.
pub const ENVELOPE_WIRE_VERSION: u8 = 1;

/// Bounded maximum size of a serialised envelope payload (1 MiB).
/// Mirrors `PAYLOAD_MAX_BYTES` in orchestrator/src/ssb.rs. Anything
/// larger is refused at the typed boundary.
pub const ENVELOPE_MAX_BYTES: usize = 1_048_576;

// ---------------------------------------------------------------------------
// MastModeWire — opaque mirror of orchestrator::mast::MastMode
// ---------------------------------------------------------------------------
//
// The full `MastMode` enum lives in `elai-orchestrator`; ipc-types
// must not depend on orchestrator (ipc-types is the lower-layer crate
// — the dependency arrow points the other way). To carry a mast tag
// across the wire without circular dep, we carry the wire-stable
// snake_case tag as `String`. Round-tripping through MastMode happens
// at the orchestrator boundary via `MastMode::from_sql_str`.

/// Opaque MAST-mode tag carried across the IPC wire as a wire-stable
/// snake_case string. Orchestrator-side callers convert via
/// `elai_orchestrator::mast::MastMode::from_sql_str`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MastTag(pub String);

// ---------------------------------------------------------------------------
// EnvelopePayload — 6 typed payload kinds
// ---------------------------------------------------------------------------

/// One of six typed payload kinds an envelope can carry. Adding a
/// new variant is an envelope-schema bump (ENVELOPE_WIRE_VERSION +
/// 1) so consumers can refuse old wire versions cleanly.
///
/// The payloads are kept INTENTIONALLY simple (mostly `String` bodies
/// + bounded scalars) to avoid coupling ipc-types to higher-level
/// types from orchestrator / elai_core. Callers serialise structured
/// data into the `body` field via JSON/postcard at the producer side
/// and parse back at the consumer side.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnvelopePayload {
    /// Executor / Researcher handoff: "here is your sub-task".
    TaskHandoff { body: String },
    /// Auditor finding-report: "I observed X".
    FindingReport { body: String },
    /// Grounding citation: "claim X is supported by source Y".
    GroundingCitation { body: String, source_url: String },
    /// Consensus vote: "I vote PASS / BLOCK on claim X" + scalar weight.
    ConsensusVote {
        claim_id: String,
        vote: String,
        weight: f64,
    },
    /// Adversary verdict: "cross-family adversary confirmed / rejected
    /// the original verdict". Mirrors the orchestrator-side
    /// AdversaryVerdict struct's wire shape.
    AdversaryVerdict {
        claim_id: String,
        cross_family_confirmed: bool,
        severity: f64,
    },
    /// MistakeBook entry: "store this lesson for future runs".
    MistakeBookEntry { baby_step_id: String, body: String },
}

impl EnvelopePayload {
    /// Wire-stable label for the payload kind. Used by the dashboard +
    /// SSB filters that surface envelope traffic without parsing the
    /// body.
    #[must_use]
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::TaskHandoff { .. } => "task_handoff",
            Self::FindingReport { .. } => "finding_report",
            Self::GroundingCitation { .. } => "grounding_citation",
            Self::ConsensusVote { .. } => "consensus_vote",
            Self::AdversaryVerdict { .. } => "adversary_verdict",
            Self::MistakeBookEntry { .. } => "mistake_book_entry",
        }
    }
}

// ---------------------------------------------------------------------------
// TypedAgentEnvelope — the wire-side struct
// ---------------------------------------------------------------------------

/// The ONLY legal payload type for inter-agent IPC writes per plan task 1.
/// Producers fill in `sender_role`, `receiver_role`, `payload`, optional
/// `mast_tag`, and `ts_ms`; consumers verify `content_hash` matches the
/// blake3 of the serialised payload + reject malformed envelopes.
///
/// Field order is wire-stable (postcard tolerates field reorder via the
/// `serde(rename)` shim but humans inspecting the wire bytes will see the
/// order below).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypedAgentEnvelope {
    /// Envelope schema version (currently `ENVELOPE_WIRE_VERSION`).
    pub envelope_v: u8,
    /// Issuing agent role. `Option` so legacy / system-level emitters can
    /// pass `None` without violating the schema.
    pub sender_role: Option<AgentRole>,
    /// Target agent role. `None` = broadcast.
    pub receiver_role: Option<AgentRole>,
    /// Typed body.
    pub payload: EnvelopePayload,
    /// Optional MAST-mode tag (wire-stable snake_case string). Used by
    /// the auditor + dashboard to filter envelopes by failure-mode
    /// hypothesis.
    pub mast_tag: Option<MastTag>,
    /// blake3 digest of the postcard-encoded payload (32 bytes). Receiver
    /// MUST recompute and compare to detect tamper / truncation.
    pub content_hash: [u8; 32],
    /// Wall-clock millis since UNIX_EPOCH at envelope creation. Producer
    /// supplies via injected ClockProvider — do NOT call SystemTime
    /// directly per entropy quarantine rule.
    pub ts_ms: i64,
}

impl TypedAgentEnvelope {
    /// Construct a new envelope with computed `content_hash`.
    ///
    /// `ts_ms` is the caller's wall-clock (millis since UNIX_EPOCH);
    /// must be supplied via the injected ClockProvider at the call
    /// site, NOT generated here (this crate has no clock dep).
    ///
    /// Returns `Err(EnvelopeError::PayloadTooLarge)` if the serialised
    /// payload exceeds `ENVELOPE_MAX_BYTES`.
    pub fn new(
        sender_role: Option<AgentRole>,
        receiver_role: Option<AgentRole>,
        payload: EnvelopePayload,
        mast_tag: Option<MastTag>,
        ts_ms: i64,
    ) -> Result<Self, EnvelopeError> {
        let payload_bytes = postcard::to_allocvec(&payload)
            .map_err(|e| EnvelopeError::SerializationFailed(e.to_string()))?;
        if payload_bytes.len() > ENVELOPE_MAX_BYTES {
            return Err(EnvelopeError::PayloadTooLarge {
                size: payload_bytes.len(),
                limit: ENVELOPE_MAX_BYTES,
            });
        }
        let content_hash: [u8; 32] = blake3::hash(&payload_bytes).into();
        Ok(Self {
            envelope_v: ENVELOPE_WIRE_VERSION,
            sender_role,
            receiver_role,
            payload,
            mast_tag,
            content_hash,
            ts_ms,
        })
    }

    /// Recompute the content hash from the current payload and check
    /// it matches the stored `content_hash`. Returns `Ok(())` on match,
    /// `Err(EnvelopeError::ContentHashMismatch)` otherwise.
    pub fn verify_content_hash(&self) -> Result<(), EnvelopeError> {
        let payload_bytes = postcard::to_allocvec(&self.payload)
            .map_err(|e| EnvelopeError::SerializationFailed(e.to_string()))?;
        let expected: [u8; 32] = blake3::hash(&payload_bytes).into();
        if expected == self.content_hash {
            Ok(())
        } else {
            Err(EnvelopeError::ContentHashMismatch)
        }
    }

    /// Serialise to wire bytes (postcard). Caller is responsible for
    /// framing (length prefix etc.) at the IpcFrame boundary.
    pub fn to_wire_bytes(&self) -> Result<Vec<u8>, EnvelopeError> {
        let bytes = postcard::to_allocvec(self)
            .map_err(|e| EnvelopeError::SerializationFailed(e.to_string()))?;
        if bytes.len() > ENVELOPE_MAX_BYTES {
            return Err(EnvelopeError::PayloadTooLarge {
                size: bytes.len(),
                limit: ENVELOPE_MAX_BYTES,
            });
        }
        Ok(bytes)
    }

    /// Parse wire bytes back into an envelope. Caller MUST call
    /// `verify_content_hash` next.
    pub fn from_wire_bytes(data: &[u8]) -> Result<Self, EnvelopeError> {
        if data.len() > ENVELOPE_MAX_BYTES {
            return Err(EnvelopeError::PayloadTooLarge {
                size: data.len(),
                limit: ENVELOPE_MAX_BYTES,
            });
        }
        let env: TypedAgentEnvelope = postcard::from_bytes(data)
            .map_err(|e| EnvelopeError::DeserializationFailed(e.to_string()))?;
        if env.envelope_v != ENVELOPE_WIRE_VERSION {
            return Err(EnvelopeError::VersionMismatch {
                expected: ENVELOPE_WIRE_VERSION,
                actual: env.envelope_v,
            });
        }
        Ok(env)
    }
}

// ---------------------------------------------------------------------------
// EnvelopeError
// ---------------------------------------------------------------------------

/// Errors surfaced by envelope construction / verification / parsing.
/// Producers and consumers alike must treat these as hard signals —
/// silent fallback to a raw-string payload is a stub-ban per plan
/// task 1 (debug-refuse mode).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EnvelopeError {
    /// postcard serialisation failed (typically OOM or unsupported
    /// payload type — should not occur in production code).
    #[error("envelope serialization failed: {0}")]
    SerializationFailed(String),
    /// postcard deserialisation failed — corrupt wire bytes or
    /// envelope version mismatch.
    #[error("envelope deserialization failed: {0}")]
    DeserializationFailed(String),
    /// Payload exceeded `ENVELOPE_MAX_BYTES`. Bounded so adversarial
    /// senders cannot exhaust orchestrator memory.
    #[error("envelope payload too large: {size} bytes exceeds limit {limit}")]
    PayloadTooLarge { size: usize, limit: usize },
    /// Schema version mismatch — receiver is older / newer than sender.
    #[error("envelope version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u8, actual: u8 },
    /// Content hash check failed — payload tampered or corrupted.
    #[error("envelope content_hash mismatch")]
    ContentHashMismatch,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_envelope() -> TypedAgentEnvelope {
        TypedAgentEnvelope::new(
            Some(AgentRole::Executor),
            Some(AgentRole::Auditor),
            EnvelopePayload::TaskHandoff {
                body: "hello".into(),
            },
            Some(MastTag("fm_1_1_disobey_task_spec".into())),
            1_700_000_000_000,
        )
        .expect("envelope construction must succeed")
    }

    #[test]
    fn wire_roundtrip_preserves_all_fields() {
        let env = make_envelope();
        let bytes = env.to_wire_bytes().expect("encode");
        let back = TypedAgentEnvelope::from_wire_bytes(&bytes).expect("decode");
        assert_eq!(env, back);
    }

    #[test]
    fn content_hash_verifies_on_unmodified_payload() {
        let env = make_envelope();
        env.verify_content_hash().expect("hash must match");
    }

    #[test]
    fn content_hash_mismatch_detected() {
        let mut env = make_envelope();
        // Mutate payload but keep stale hash.
        env.payload = EnvelopePayload::TaskHandoff {
            body: "tampered".into(),
        };
        match env.verify_content_hash() {
            Err(EnvelopeError::ContentHashMismatch) => {}
            other => panic!("expected ContentHashMismatch, got {other:?}"),
        }
    }

    #[test]
    fn payload_kind_str_stable() {
        let p = EnvelopePayload::ConsensusVote {
            claim_id: "c".into(),
            vote: "pass".into(),
            weight: 1.0,
        };
        assert_eq!(p.kind_str(), "consensus_vote");
    }

    #[test]
    fn version_mismatch_is_detected() {
        let mut env = make_envelope();
        env.envelope_v = 99;
        let bytes = env.to_wire_bytes().expect("encode");
        match TypedAgentEnvelope::from_wire_bytes(&bytes) {
            Err(EnvelopeError::VersionMismatch { .. }) => {}
            other => panic!("expected VersionMismatch, got {other:?}"),
        }
    }

    #[test]
    fn six_payload_kinds_each_round_trip() {
        let ts = 1_700_000_000_000;
        let payloads = [
            EnvelopePayload::TaskHandoff { body: "t".into() },
            EnvelopePayload::FindingReport { body: "f".into() },
            EnvelopePayload::GroundingCitation {
                body: "g".into(),
                source_url: "https://example.org".into(),
            },
            EnvelopePayload::ConsensusVote {
                claim_id: "c".into(),
                vote: "block".into(),
                weight: 0.7,
            },
            EnvelopePayload::AdversaryVerdict {
                claim_id: "c".into(),
                cross_family_confirmed: true,
                severity: 0.9,
            },
            EnvelopePayload::MistakeBookEntry {
                baby_step_id: "bs".into(),
                body: "m".into(),
            },
        ];
        for p in payloads {
            let env = TypedAgentEnvelope::new(None, None, p, None, ts).expect("encode");
            let bytes = env.to_wire_bytes().expect("wire");
            let back = TypedAgentEnvelope::from_wire_bytes(&bytes).expect("decode");
            assert_eq!(env, back);
        }
    }
}
