//! D-10 — typed `Claim` value object + closed [`ClaimType`] taxonomy.
//!
//! Pure data module — no I/O, no async, no LLM calls. The mode-routing
//! predicate ([`crate::grounding_mode::select_grounding_mode`]) reads
//! a `Claim` + an [`crate::orch_context::OrchContext`] and produces a
//! [`crate::grounding_mode::GroundingMode`] deterministically.
//!
//! Grounding:
//!   * docs/specs/guide/real-problem-solving-grounded-2026-05-19.md §1
//!     (5-mode taxonomy) + §3.2 (routing predicate).
//!   * arxiv:1908.04734 (Everitt current-RF principle, T1) — agent must
//!     not edit the grading function; `ClaimType` is a closed enum the
//!     L3 frozen-surface registry (D-9) locks.
//!   * arxiv:1906.01820 (Hubinger mesa-optimization).

use blake3::Hash;
use serde::{Deserialize, Serialize};

/// Closed taxonomy of claim kinds the orchestrator routes on. Adding a
/// variant requires a plan-level schema bump; the agent cannot extend
/// it at runtime (L3-locked via the locked-keys registry).
///
/// Mapping summary (full table in workflow/reasoning/D-10-decisions.md):
///   * `Factual`       → constructive checker preferred.
///   * `Procedural`    → operational-falsification preferred.
///   * `Normative`     → conservative-fallback (no checker can ground "ought").
///   * `Speculative`   → falsifiability-cross-family if signals support it,
///                       else conformal-refusal.
///   * `Hypothetical`  → operational-deferred (sandbox experiment when ready).
///   * `Computational` → constructive (checker / sympy / runner).
///   * `MetaQuery`     → conservative-fallback (orchestrator-level reflection
///                       is not a graded claim).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimType {
    Factual,
    Procedural,
    Normative,
    Speculative,
    Hypothetical,
    Computational,
    MetaQuery,
}

impl ClaimType {
    /// Stable snake-case wire label used in SSB rows + dashboard
    /// rendering. Matches the `#[serde(rename_all = "snake_case")]`
    /// derive so the SSB payload deserializes round-trip.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Factual => "factual",
            Self::Procedural => "procedural",
            Self::Normative => "normative",
            Self::Speculative => "speculative",
            Self::Hypothetical => "hypothetical",
            Self::Computational => "computational",
            Self::MetaQuery => "meta_query",
        }
    }
}

/// Origin/kind of an evidence record bound to a claim. Closed enum —
/// the auditor cannot accept evidence from arbitrary sources at
/// runtime (defends against "I have a memory of X" handwaving).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    /// Citation into the project corpus (file path + line range hash).
    Corpus,
    /// Captured output of a deterministic tool call (compiler / test /
    /// linter / SCIP query etc.).
    ToolOutput,
    /// Derivation step in the current reasoning chain (mcp_think row).
    Derivation,
    /// Long-term memory fact (E-5 `memory_facts` store).
    MemoryFact,
}

impl EvidenceKind {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Corpus => "corpus",
            Self::ToolOutput => "tool_output",
            Self::Derivation => "derivation",
            Self::MemoryFact => "memory_fact",
        }
    }
}

/// Single piece of evidence attached to a claim. The `id` is opaque
/// (path / row uuid / hash) — the auditor uses (kind, id) as the
/// dedup key and `blake3` to detect post-hoc tampering of the
/// evidence content.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub kind: EvidenceKind,
    /// Opaque content-addressable identifier for the evidence (path,
    /// uuid, registry row, etc.). Kept as `String` for IPC stability.
    pub id: String,
    /// blake3 hash of the canonical pre-image — 32-byte array exposed
    /// over the wire as base64-less raw bytes (msgpack handles binary
    /// natively; JSON gets a byte array).
    #[serde(with = "serde_bytes_32")]
    pub blake3: [u8; 32],
}

impl EvidenceRef {
    /// Construct an `EvidenceRef` by hashing `content_bytes` with
    /// blake3. Centralises the hash invocation so producers cannot
    /// accidentally use blake2 / sha2 (anti-stale-training rule).
    #[must_use]
    pub fn from_content(kind: EvidenceKind, id: impl Into<String>, content_bytes: &[u8]) -> Self {
        let h: Hash = blake3::hash(content_bytes);
        Self {
            kind,
            id: id.into(),
            blake3: *h.as_bytes(),
        }
    }
}

/// Typed claim emitted by an agent (or the orchestrator itself) prior
/// to mode routing. Fields kept minimal — payload-level metadata
/// (provider, model, prompt hash) is recorded on the SSB row, not
/// duplicated here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Claim {
    /// Stable per-claim identifier. Producers normally generate a uuid;
    /// kept as `String` so the type is portable across IPC.
    pub claim_id: String,
    /// Bounded plaintext statement; producers must truncate before
    /// constructing the `Claim` (no truncation here — the router does
    /// not look at `text`, only at `claim_type` + `evidence`).
    pub text: String,
    pub claim_type: ClaimType,
    pub evidence: Vec<EvidenceRef>,
    /// Wall-clock UNIX ms when the claim was emitted. `i64` (not
    /// `u64`) matches the SSB schema's `ts_ms` widening.
    pub ts_ms: i64,
}

impl Claim {
    /// True iff the claim has at least one piece of corpus or tool
    /// evidence — the minimum bar for `select_grounding_mode` to even
    /// consider a constructive verifier. Used by `Speculative` claims
    /// to decide between cross-family + refusal.
    #[must_use]
    pub fn has_grounding_evidence(&self) -> bool {
        self.evidence.iter().any(|e| {
            matches!(
                e.kind,
                EvidenceKind::Corpus | EvidenceKind::ToolOutput | EvidenceKind::Derivation
            )
        })
    }
}

/// Serde helper for the 32-byte blake3 hash. Msgpack serializes it as
/// a fixed binary blob; JSON falls back to a sequence of `u8`s which
/// is the only stable cross-language representation (Rust's
/// `serde_bytes` crate would also work but we avoid adding a
/// dependency).
mod serde_bytes_32 {
    use serde::de::{Error as DeError, SeqAccess, Visitor};
    use serde::ser::SerializeSeq;
    use serde::{Deserializer, Serializer};
    use std::fmt;

    pub fn serialize<S>(bytes: &[u8; 32], ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = ser.serialize_seq(Some(32))?;
        for b in bytes {
            seq.serialize_element(b)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(de: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = [u8; 32];
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a 32-byte blake3 hash as a byte sequence")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut out = [0u8; 32];
                let mut i = 0usize;
                while let Some(b) = seq.next_element::<u8>()? {
                    if i >= 32 {
                        return Err(DeError::invalid_length(i + 1, &self));
                    }
                    out[i] = b;
                    i += 1;
                }
                if i != 32 {
                    return Err(DeError::invalid_length(i, &self));
                }
                Ok(out)
            }
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                if v.len() != 32 {
                    return Err(DeError::invalid_length(v.len(), &self));
                }
                let mut out = [0u8; 32];
                out.copy_from_slice(v);
                Ok(out)
            }
            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                self.visit_bytes(v)
            }
        }
        de.deserialize_seq(V)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_evidence(kind: EvidenceKind, id: &str) -> EvidenceRef {
        EvidenceRef::from_content(kind, id, id.as_bytes())
    }

    #[test]
    fn claim_type_as_str_round_trip_matches_serde() {
        // Each variant's `as_str` MUST equal its serde tag so the SSB
        // payload (which uses #[serde(rename_all = "snake_case")])
        // round-trips through ssb_viewer JSON.
        for ct in [
            ClaimType::Factual,
            ClaimType::Procedural,
            ClaimType::Normative,
            ClaimType::Speculative,
            ClaimType::Hypothetical,
            ClaimType::Computational,
            ClaimType::MetaQuery,
        ] {
            let json = serde_json::to_string(&ct).expect("json serialize");
            let stripped = json.trim_matches('"');
            assert_eq!(stripped, ct.as_str(), "as_str / serde mismatch for {ct:?}");
        }
    }

    #[test]
    fn evidence_ref_blake3_is_stable() {
        let a = EvidenceRef::from_content(EvidenceKind::Corpus, "file.rs:10-20", b"hello");
        let b = EvidenceRef::from_content(EvidenceKind::Corpus, "file.rs:10-20", b"hello");
        assert_eq!(a.blake3, b.blake3, "blake3 must be deterministic on same input");
    }

    #[test]
    fn has_grounding_evidence_requires_real_source() {
        let mut c = Claim {
            claim_id: "c1".into(),
            text: "x".into(),
            claim_type: ClaimType::Factual,
            evidence: vec![],
            ts_ms: 1_700_000_000_000,
        };
        assert!(!c.has_grounding_evidence(), "empty claim is ungrounded");
        c.evidence.push(mk_evidence(EvidenceKind::MemoryFact, "mem-1"));
        assert!(
            !c.has_grounding_evidence(),
            "memory-fact alone is not grounding evidence — D-10 §3.2"
        );
        c.evidence.push(mk_evidence(EvidenceKind::Corpus, "src/main.rs:1-10"));
        assert!(c.has_grounding_evidence(), "corpus evidence grounds the claim");
    }

    #[test]
    fn claim_roundtrips_through_json() {
        let c = Claim {
            claim_id: "c-abc".into(),
            text: "the renderer batches draw calls".into(),
            claim_type: ClaimType::Factual,
            evidence: vec![mk_evidence(EvidenceKind::Corpus, "src/render.rs:42-50")],
            ts_ms: 1_700_000_000_000,
        };
        let s = serde_json::to_string(&c).expect("serialize");
        let c2: Claim = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(c, c2);
    }
}

// L-1: typed claim EXTRACTION submodule. Different axis from the D-10 `ClaimType`
// taxonomy above: D-10 types what a claim MEANS, `extract` types what it REFERS TO.
// Composed, never merged (plan L-1 GD-1).
pub mod extract;
pub mod prune;
pub mod resolve;
pub mod risk;
pub mod roundtrip;
pub mod semantic_entropy;
pub use extract::{extract_claims, ClaimKind, ClaimSet, ExtractedClaim};
