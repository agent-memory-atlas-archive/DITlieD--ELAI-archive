//! G-8 — Signed in-toto v1 provenance attestation (the G-plan trust-spine
//! off-machine extension).
//!
//! G-2 ([`crate::rra`]) sealed a run's evidence INSIDE the machine: the action
//! stream, the determinism pins, the F-gate verdict manifest, a blake3 root. F-4
//! bound an outcome to its artifact INSIDE the run. G-8 extends that trust OFF
//! the machine: it re-expresses the G-2 [`RunArtifact`] as a custom
//! [in-toto v1 Statement] + predicate, signs it with a LOCAL hand-rolled
//! blake3-Merkle proof (the DEFAULT, no new dependency), and emits it to three
//! targets so anyone can re-verify the run OFFLINE without re-running ELAI
//! ("trust without re-running", the SLSA-VSA model):
//!   * a `provenance_attestation_emitted` SSB row (the off-agent beacon);
//!   * a sidecar `attestation.intoto.json` (the full Statement);
//!   * a digest-only git commit trailer (`ELAI-Provenance-Digest: <root>`).
//!
//! GD-1 (COMPOSE, re-implement none): [`build_attestation`] CONSUMES the G-2
//! [`RunArtifact`] manifest verbatim — the predicate's `verified_levels` IS the
//! G-2 [`GateVerdictHash`] manifest (`ssb_event_seq:i64`, the G-2-locked field),
//! re-expressed as an SLSA-VSA PASSED rollup. It re-derives NO F-gate.
//!
//! GD-2 (byte-identical recompute): the Merkle root is blake3 over the canonical
//! predicate bytes (`rmp_serde::to_vec_named` of a fixed-order tuple), mirroring
//! [`crate::rra::compute_root`] EXACTLY. The same predicate always produces the
//! same root, so a verifier recomputes it byte-identically (under the pinned
//! `ELAI_SEED`). A single-byte tamper (a flipped verdict / hash) changes the
//! canonical bytes and the root no longer recomputes — a NOT-VERIFIED verdict,
//! never a silent pass (R2/R4, mirrors [`crate::rra::verify_artifact`] +
//! [`crate::archive_crypto::placeholder_verify`]).
//!
//! THE HONEST SIGNING MODEL (GD-5). The DEFAULT signature is a LOCAL blake3-Merkle
//! proof: it proves the predicate has not been tampered with (offline recompute-
//! and-compare), NOT that a transparency log (Sigstore Rekor) witnessed it. There
//! is no keyless / third-party witness in the default path. A cosign-keyless
//! upgrade (Sigstore + Rekor transparency, ed25519/sigstore-rs) is a NEW
//! DEPENDENCY and is DEFERRED per CLAUDE.md no-new-deps — it ships behind
//! `#[cfg(feature = "cosign-attest")]` and the crate MUST cargo-check-green
//! WITHOUT the feature. See [`SIGNING_MODEL_DEFERRED_COSIGN`].
//!
//! ARTICLE-50 HONESTY (DD6, E-38 HARD RULE). This attestation is "designed to
//! support EU AI Act Article-50 transparency marking", NOT "certified compliant"
//! (the final GPAI Code of Practice is ~June 2026; source code is not an
//! enumerated Art-50 example). The relevant penalty tier is Article-99 Tier-2:
//! up to EUR 15M or 3% of worldwide annual turnover (the transparency-obligation
//! tier covering Art-50), NEVER the Art-5 prohibited-practices tier (EUR 35M /
//! 7%). See [`ARTICLE_50_POSTURE`]; figure grounded against
//! artificialintelligenceact.eu/article/99 (Tier-2, verified 2026-06-04) +
//! docs/strategy/ELAI-2026-POSTURE.md Section 5.
//!
//! blake3 is the only hash (CLAUDE.md hard rule). No new dependency: the bundle
//! is serialized with `rmp_serde` (already a direct orchestrator dep) and the
//! sidecar with `serde_json` (likewise).
//!
//! [in-toto v1 Statement]: https://in-toto.io/Statement/v1

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::rra::{GateVerdictHash, RunArtifact, RRA_HEX_PREFIX_LEN};

/// The in-toto v1 Statement `_type` URI (the in-toto standard envelope type).
pub const IN_TOTO_STATEMENT_TYPE: &str = "https://in-toto.io/Statement/v1";

/// The custom ELAI predicate type URI. in-toto v1 permits arbitrary custom
/// predicate types; this is ELAI's stable provenance-attestation predicate URI.
pub const ELAI_PREDICATE_TYPE: &str = "https://elai.dev/attestation/run-provenance/v1";

/// The subject-digest algorithm key in the in-toto `digest` map. blake3 is the
/// workspace default (CLAUDE.md hard rule); `sha256` is reserved for the
/// deferred cosign-interop feature only.
pub const SUBJECT_DIGEST_ALG: &str = "blake3";

/// Documented deferred enhancement (DD1, CLAUDE.md no-new-deps): the default
/// signing path is a LOCAL hand-rolled blake3-Merkle proof (recompute-and-compare
/// offline). A cosign-keyless upgrade (Sigstore + Rekor transparency log,
/// `sigstore-rs` / `ed25519-dalek`) is a NEW dependency and is DEFERRED; it ships
/// behind `cfg(feature = "cosign-attest")` and the crate compiles green WITHOUT
/// the feature. Carried here so the decision is auditable.
pub const SIGNING_MODEL_DEFERRED_COSIGN: &str =
    "local blake3-Merkle proof (offline recompute); cosign-keyless/Sigstore-Rekor deferred \
     (new dep; promotion needs confirmation + anti-slopsquat verification)";

/// The honest EU AI Act posture string (DD6, E-38). Article-50 transparency
/// marking is the SUPPORTED use; the relevant penalty tier is Article-99 Tier-2
/// (EUR 15M / 3% worldwide turnover), NOT the Art-5 prohibited-practices tier
/// (EUR 35M / 7%). NEVER asserts "certified compliant".
pub const ARTICLE_50_POSTURE: &str =
    "designed to support EU AI Act Article-50 transparency marking (Article-99 Tier-2: \
     up to EUR 15M or 3% of worldwide annual turnover); NOT certified compliant";

/// One verified-level entry in the predicate's SLSA-VSA rollup.
///
/// REUSE-DON'T-FORK (GD-1): this embeds the G-2 [`GateVerdictHash`] verbatim
/// (its `ssb_event_seq:i64` field is G-2-locked) and adds a derived `passed`
/// view (a present manifest entry IS a passed gate — a gate that was off / silent
/// leaves NO entry, a valid gate-off state, so every entry here is `true`). The
/// hash type is NOT re-declared; only the `passed` boolean is layered on top.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedLevel {
    /// The G-2 per-gate verdict hash, reused verbatim (GD-1).
    #[serde(flatten)]
    pub gate_hash: GateVerdictHash,
    /// SLSA-VSA `verificationResult` view: `true` because a present manifest
    /// entry is a gate that fired and recorded a verdict. A gate-off state
    /// leaves no entry at all (it is never recorded as `false`).
    pub passed: bool,
}

/// The custom ELAI provenance predicate (the in-toto Statement `predicate` body).
///
/// Carries the run's model identity, the pinned `ELAI_SEED`, the SLSA-VSA
/// `verified_levels` rollup (the G-2 F-gate manifest re-expressed, GD-1), the
/// evidence hashes (the G-2 root + the per-gate verdict hashes flattened for a
/// quick scan), and the SSB event seq the run-artifact beacon landed at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InTotoPredicate {
    /// The run's model identity (the G-2 `model_pin`); `None` when no model
    /// swap was recorded in the stream.
    pub model_id: Option<String>,
    /// The pinned `ELAI_SEED` the action stream re-derives under (GD-2).
    pub elai_seed: u64,
    /// The honest per-lane guarantee label (the G-2 `guarantee_level`).
    pub guarantee_level: String,
    /// The SLSA-VSA rollup of every F-gate verdict PRESENT in the run (GD-1:
    /// CONSUMED from the G-2 manifest, re-implements none).
    pub verified_levels: Vec<VerifiedLevel>,
    /// The flattened evidence hashes (the G-2 bundle root + every per-gate
    /// verdict hash) for a quick scan without walking `verified_levels`.
    pub evidence_hashes: Vec<String>,
    /// The G-2 bundle root the attestation chains to (the run-artifact anchor).
    pub run_artifact_root: String,
    /// The honest EU AI Act posture string (DD6, E-38). See [`ARTICLE_50_POSTURE`].
    pub article_50_posture: String,
}

/// One in-toto v1 subject (what the attestation is ABOUT).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InTotoSubject {
    /// The subject name (the session id; the run's logical identity). NOT the
    /// commit SHA (the subject digests the staged tree state, not a not-yet-
    /// existing commit — chicken-and-egg, stub_ban DD).
    pub name: String,
    /// The subject digest map: `{ "blake3": <merkle-root> }`. The root is the
    /// blake3-Merkle proof over the canonical predicate bytes (GD-2).
    pub digest: std::collections::BTreeMap<String, String>,
}

/// The full in-toto v1 Statement: subject + predicate type + predicate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InTotoStatement {
    /// The in-toto Statement envelope type URI.
    #[serde(rename = "_type")]
    pub statement_type: String,
    /// What the attestation is about (the run, identified by session id).
    pub subject: Vec<InTotoSubject>,
    /// The custom ELAI predicate type URI.
    #[serde(rename = "predicateType")]
    pub predicate_type: String,
    /// The custom ELAI provenance predicate body.
    pub predicate: InTotoPredicate,
}

impl InTotoStatement {
    /// The subject digest (the blake3-Merkle root) the statement is signed with.
    /// `None` only on the impossible empty-subject path (the constructor always
    /// pushes exactly one subject).
    #[must_use]
    pub fn subject_digest(&self) -> Option<&str> {
        self.subject
            .first()
            .and_then(|s| s.digest.get(SUBJECT_DIGEST_ALG))
            .map(String::as_str)
    }
}

/// Errors from building / writing / verifying an attestation.
#[derive(Debug, Error)]
pub enum AttestationError {
    /// Sidecar file I/O failure.
    #[error("attestation io error at {path}: {detail}")]
    Io { path: String, detail: String },
    /// msgpack canonical-bytes (de)serialization failure (the signing path).
    #[error("attestation msgpack error: {0}")]
    Msgpack(String),
    /// JSON sidecar (de)serialization failure (the human-readable sidecar).
    #[error("attestation json error: {0}")]
    Json(String),
}

/// The offline verify verdict (mirrors [`crate::rra::ReplayVerdict`] posture).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "verdict")]
pub enum AttestationVerdict {
    /// The blake3-Merkle root recomputes over the predicate AND equals the
    /// subject digest (and the trailer digest, when one is supplied).
    Verified {
        /// The blake3-Merkle root that recomputed (== the subject digest).
        merkle_root: String,
    },
    /// The recomputed root does NOT equal the subject / trailer digest — the
    /// predicate was tampered (a flipped verdict / hash / seed). NEVER a silent
    /// pass (R2): missing / mismatched evidence is reported.
    NotVerified {
        /// The digest the statement claims (the subject / trailer digest).
        expected: String,
        /// The root recomputed over the predicate now.
        got: String,
    },
}

/// Truncate a full blake3 hex to the workspace 16-hex prefix convention
/// (matches [`crate::rra`]'s `RRA_HEX_PREFIX_LEN`).
fn truncate_hex(full_hex: &str) -> String {
    full_hex.chars().take(RRA_HEX_PREFIX_LEN).collect()
}

/// Compute the canonical predicate bytes: the msgpack of a deterministic tuple
/// in a FIXED order. The same predicate always produces the same bytes, so the
/// Merkle root recomputes byte-identically (GD-2). Mirrors
/// [`crate::rra`]'s `canonical_bundle_bytes` EXACTLY (rmp_serde::to_vec_named of
/// a fixed-order tuple), NOT `serde_json` — JSON map-key ordering is not
/// guaranteed stable across serde versions and would break GD-2.
fn canonical_predicate_bytes(predicate: &InTotoPredicate) -> Result<Vec<u8>, AttestationError> {
    // Sort verified_levels by (gate, ssb_event_seq) for stable ordering
    // independent of scan order (mirrors rra::canonical_bundle_bytes:276).
    let mut levels = predicate.verified_levels.clone();
    levels.sort_by(|a, b| {
        a.gate_hash
            .gate
            .cmp(&b.gate_hash.gate)
            .then(a.gate_hash.ssb_event_seq.cmp(&b.gate_hash.ssb_event_seq))
    });
    // evidence_hashes is sorted for the same stable-ordering reason.
    let mut evidence = predicate.evidence_hashes.clone();
    evidence.sort();
    let canonical = (
        &predicate.model_id,
        predicate.elai_seed,
        &predicate.guarantee_level,
        &levels,
        &evidence,
        &predicate.run_artifact_root,
        &predicate.article_50_posture,
    );
    rmp_serde::to_vec_named(&canonical).map_err(|e| AttestationError::Msgpack(e.to_string()))
}

/// Sign the predicate with the LOCAL blake3-Merkle proof: the blake3 root
/// (16-hex prefix) over the canonical predicate bytes. The canonical bytes
/// already commit to EVERY field — including every F-gate verdict hash and the
/// seed — so the root binds them all (a flipped verdict re-hashes to a different
/// root). This is the no-new-dep default; mirrors [`crate::rra::compute_root`] +
/// the [`crate::archive_crypto::placeholder_sign`] recompute-and-compare posture.
pub fn sign_local_merkle(predicate: &InTotoPredicate) -> Result<String, AttestationError> {
    let bytes = canonical_predicate_bytes(predicate)?;
    Ok(truncate_hex(&blake3::hash(&bytes).to_hex()))
}

/// Build a signed in-toto v1 [`InTotoStatement`] from a G-2 [`RunArtifact`].
///
/// GD-1 (COMPOSE): CONSUMES `art.manifest` (the F-gate verdict hashes), the
/// `art.root_blake3`, `art.seed`, `art.model_pin`, `art.guarantee_level`. It
/// re-derives NO F-gate. The subject is the run's `session_id` (NOT a commit
/// SHA — the subject digests the run's evidence, chicken-and-egg with the commit,
/// stub_ban DD). The subject digest is the blake3-Merkle root over the predicate.
pub fn build_attestation(art: &RunArtifact) -> Result<InTotoStatement, AttestationError> {
    // verified_levels = the G-2 manifest re-expressed as an SLSA-VSA rollup
    // (GD-1: each present entry is a gate that fired and recorded a verdict).
    let verified_levels: Vec<VerifiedLevel> = art
        .manifest
        .iter()
        .map(|gh| VerifiedLevel {
            gate_hash: gh.clone(),
            passed: true,
        })
        .collect();

    // evidence_hashes = the bundle root + every per-gate verdict hash, flattened
    // for a quick scan. Sorted in canonical_predicate_bytes for GD-2 stability.
    let mut evidence_hashes: Vec<String> = Vec::with_capacity(art.manifest.len() + 1);
    evidence_hashes.push(art.root_blake3.clone());
    for gh in &art.manifest {
        evidence_hashes.push(gh.verdict_blake3.clone());
    }

    let predicate = InTotoPredicate {
        model_id: art.model_pin.clone(),
        elai_seed: art.seed,
        guarantee_level: art.guarantee_level.as_sql_str().to_owned(),
        verified_levels,
        evidence_hashes,
        run_artifact_root: art.root_blake3.clone(),
        article_50_posture: ARTICLE_50_POSTURE.to_owned(),
    };

    // Sign: the subject digest IS the blake3-Merkle root over the predicate.
    let merkle_root = sign_local_merkle(&predicate)?;

    let mut digest = std::collections::BTreeMap::new();
    digest.insert(SUBJECT_DIGEST_ALG.to_owned(), merkle_root);

    Ok(InTotoStatement {
        statement_type: IN_TOTO_STATEMENT_TYPE.to_owned(),
        subject: vec![InTotoSubject {
            name: art.session_id.clone(),
            digest,
        }],
        predicate_type: ELAI_PREDICATE_TYPE.to_owned(),
        predicate,
    })
}

/// Verify an attestation OFFLINE (trust-without-re-running, SLSA-VSA): recompute
/// the blake3-Merkle root over the predicate and compare it to the statement's
/// subject digest. A flipped verdict / hash / seed re-hashes to a different root
/// -> [`AttestationVerdict::NotVerified`]. Does NOT re-run any F-gate. Mirrors
/// [`crate::rra::verify_artifact`]: the verdict is a pure property of the
/// predicate bytes the verifier holds, never a fabricated pass (R2/R4).
pub fn verify_attestation(
    statement: &InTotoStatement,
) -> Result<AttestationVerdict, AttestationError> {
    let recomputed = sign_local_merkle(&statement.predicate)?;
    let expected = statement.subject_digest().unwrap_or("");
    if recomputed == expected && !expected.is_empty() {
        Ok(AttestationVerdict::Verified {
            merkle_root: recomputed,
        })
    } else {
        Ok(AttestationVerdict::NotVerified {
            expected: expected.to_owned(),
            got: recomputed,
        })
    }
}

/// Verify an attestation OFFLINE AND confirm it equals an external trailer digest
/// (the git-binding leg of `elai verify-attestation`). Recomputes the Merkle root
/// over the predicate, requires it to equal BOTH the subject digest AND the
/// supplied trailer digest. A mismatch on either side is [`AttestationVerdict::NotVerified`]
/// (never a fabricated pass). Does NOT re-run any F-gate.
pub fn verify_attestation_against_trailer(
    statement: &InTotoStatement,
    trailer_digest: &str,
) -> Result<AttestationVerdict, AttestationError> {
    let recomputed = sign_local_merkle(&statement.predicate)?;
    let subject = statement.subject_digest().unwrap_or("");
    if !subject.is_empty() && recomputed == subject && recomputed == trailer_digest {
        Ok(AttestationVerdict::Verified {
            merkle_root: recomputed,
        })
    } else {
        // Report the binding the verify failed against: the trailer digest is the
        // external claim, the recomputed root is what the predicate actually
        // hashes to. NEVER swallow a mismatch (R2).
        Ok(AttestationVerdict::NotVerified {
            expected: trailer_digest.to_owned(),
            got: recomputed,
        })
    }
}

/// Serialize the in-toto Statement to a sidecar `attestation.intoto.json`
/// (human-readable JSON, mirrors [`crate::rra::write_rra`]'s create_dir_all +
/// fs::write shape but JSON not msgpack).
pub fn write_sidecar(
    statement: &InTotoStatement,
    path: &std::path::Path,
) -> Result<(), AttestationError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AttestationError::Io {
            path: parent.display().to_string(),
            detail: e.to_string(),
        })?;
    }
    let bytes = serde_json::to_vec_pretty(statement)
        .map_err(|e| AttestationError::Json(e.to_string()))?;
    std::fs::write(path, &bytes).map_err(|e| AttestationError::Io {
        path: path.display().to_string(),
        detail: e.to_string(),
    })
}

/// Read an in-toto Statement back from a sidecar `attestation.intoto.json`.
pub fn read_sidecar(path: &std::path::Path) -> Result<InTotoStatement, AttestationError> {
    let bytes = std::fs::read(path).map_err(|e| AttestationError::Io {
        path: path.display().to_string(),
        detail: e.to_string(),
    })?;
    serde_json::from_slice::<InTotoStatement>(&bytes)
        .map_err(|e| AttestationError::Json(e.to_string()))
}

// ---------------------------------------------------------------------------
// Cosign-keyless opt-in (DEFERRED, feature-gated, dep-free without the feature).
// ---------------------------------------------------------------------------

/// Cosign-keyless signing is the deferred Sigstore + Rekor transparency upgrade
/// (DD1). It is a NEW DEPENDENCY (`sigstore-rs` / `ed25519-dalek`) and ships ONLY
/// behind `cfg(feature = "cosign-attest")`. The crate MUST cargo-check-green
/// WITHOUT the feature — so this is a documentation-only marker today; the dep is
/// NOT added to Cargo.toml. When the feature is approved + the dep anti-slopsquat-
/// verified, the keyless signer body lands here behind the gate.
#[cfg(feature = "cosign-attest")]
pub const COSIGN_FEATURE_NOTE: &str =
    "cosign-keyless (Sigstore + Rekor) enabled; the default local blake3-Merkle proof is \
     supplemented by a transparency-log witness. Requires the cosign-attest feature + \
     an anti-slopsquat-verified sigstore-rs dependency.";
