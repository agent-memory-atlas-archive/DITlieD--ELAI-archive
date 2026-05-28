//! E-7: Archive chain crypto extension (Ed25519 RUNPOD-PENDING).
//!
//! Per E-7 plan task 9: each ArchiveEntry gains
//! {parent_lock_list_blake3, verifier_set_blake3, ed25519_sig} fields.
//! Second-party signer = auditor agent process (separate
//! `AgentRole::Auditor` process per Tier-C isolation). `elai archive
//! verify` walks the chain and verifies signatures.
//!
//! Paper grounding:
//!   - arXiv:1908.04734 (Everitt reward tampering) — grounds the
//!     second-party signer requirement (RF-input + reward-function
//!     tampering threat model).
//!
//! ANTI-SLOPSQUATTING: ed25519-dalek crate dependency is NOT added
//! this session. Per CLAUDE.md hard rule, every new dep requires
//! `cargo info ed25519-dalek` + repo verification + license check.
//! RustCrypto/signatures is canonical for ed25519-dalek; verification
//! deferred to a follow-on patch per
//! workflow/reasoning/E-7-decisions.md DECISION-15.
//!
//! This module ships a BLAKE3-ONLY placeholder for the signature
//! field. The interface signature uses an opaque `[u8; 64]` sig type
//! that matches Ed25519's 64-byte output, so the upgrade is a drop-in
//! replacement when ed25519-dalek is approved.
//!
//! Wire-dark: compiles only when `cfg(feature = "darwinian-evolve")`
//! is set.

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Ed25519 signature length (bytes). Matches RustCrypto/signatures
/// ed25519::Signature::BYTE_SIZE = 64. Reserved here so the upgrade
/// from the blake3 placeholder to real ed25519 is shape-stable.
pub const ED25519_SIG_LEN: usize = 64;

/// Archive link with the E-7 extension fields. The
/// `ed25519_sig` field is a `[u8; 64]` opaque placeholder until the
/// ed25519-dalek dep verification clears anti-slopsquatting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ed25519SignedLink {
    /// blake3 of the canonical link payload.
    pub self_blake3: [u8; 32],
    /// blake3 of the parent link payload (chain continuity).
    pub parent_blake3: [u8; 32],
    /// blake3 of the parent's lock-list state (catches lock-list edits).
    pub parent_lock_list_blake3: [u8; 32],
    /// blake3 of the verifier-set definitions (catches verifier-set
    /// edits per D-3 invariant).
    pub verifier_set_blake3: [u8; 32],
    /// Ed25519 signature over (self_blake3 || parent_blake3 ||
    /// parent_lock_list_blake3 || verifier_set_blake3). RUNPOD-PENDING:
    /// currently a 64-byte blake3-extended digest (NOT a real signature)
    /// pending ed25519-dalek anti-slopsquatting verification.
    #[serde(with = "serde_arrays_sig")]
    pub ed25519_sig: [u8; ED25519_SIG_LEN],
}

/// Compute the canonical payload bytes signed by `ed25519_sig`.
/// Stable ordering: self || parent || parent_lock_list || verifier_set.
#[must_use]
pub fn canonical_payload(link: &Ed25519SignedLink) -> [u8; 128] {
    let mut buf = [0u8; 128];
    buf[0..32].copy_from_slice(&link.self_blake3);
    buf[32..64].copy_from_slice(&link.parent_blake3);
    buf[64..96].copy_from_slice(&link.parent_lock_list_blake3);
    buf[96..128].copy_from_slice(&link.verifier_set_blake3);
    buf
}

/// Compute the BLAKE3-placeholder signature for a link payload.
/// RUNPOD-PENDING: replace with `ed25519_dalek::SigningKey::sign` when
/// the dep is approved per DECISION-15.
/// Dep verification 2026-05-29 (offline batch s-25120):
///   crates.io ed25519-dalek max_stable=2.2.0 max=3.0.0-pre.7
///   recent_downloads=48,239,505 owner=dalek-cryptography updated=2026-05-06
///   homepage=github.com/dalek-cryptography/curve25519-dalek
///   lib.rs reports ~20.7M dl/mo. License (per crates.io): BSD-3-Clause
///   (verify exact spdx in Cargo.toml when adding). Slopsquat check PASS.
///   READY FOR USER APPROVAL: add `ed25519-dalek = "2.2"` to
///   crates/orchestrator/Cargo.toml + replace this fn body.
#[must_use]
pub fn placeholder_sign(payload: &[u8]) -> [u8; ED25519_SIG_LEN] {
    let mut hasher = Hasher::new();
    hasher.update(b"E-7 RUNPOD-PENDING placeholder; replace with ed25519-dalek");
    hasher.update(payload);
    let digest = hasher.finalize();
    let mut sig = [0u8; ED25519_SIG_LEN];
    // blake3 emits 32 bytes; extend to 64 by double-feeding with a domain
    // separator so the placeholder differs structurally from a raw hash.
    sig[0..32].copy_from_slice(digest.as_bytes());
    let mut hasher2 = Hasher::new();
    hasher2.update(b"E-7 RUNPOD-PENDING placeholder second-half");
    hasher2.update(digest.as_bytes());
    sig[32..64].copy_from_slice(hasher2.finalize().as_bytes());
    sig
}

/// Verify the BLAKE3-placeholder signature. RUNPOD-PENDING: replace
/// with `ed25519_dalek::VerifyingKey::verify` when dep approved.
pub fn placeholder_verify(
    payload: &[u8],
    sig: &[u8; ED25519_SIG_LEN],
) -> Result<(), CryptoError> {
    let expected = placeholder_sign(payload);
    if expected == *sig {
        Ok(())
    } else {
        Err(CryptoError::SignatureBroken)
    }
}

/// Archive crypto failure modes.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CryptoError {
    #[error("ed25519 signature broken (RUNPOD-PENDING: placeholder mismatch)")]
    SignatureBroken,
    #[error("lock-list hash mismatch -- parent lock-list was edited")]
    LockListHashMismatch,
}

// serde shim for [u8; 64] (serde does not derive arrays > 32 by default
// in some older serde minor versions; explicit shim keeps this stable
// across workspace serde bumps).
mod serde_arrays_sig {
    use serde::{
        de::{self, Visitor},
        Deserializer, Serializer,
    };
    use std::fmt;

    pub fn serialize<S>(arr: &[u8; super::ED25519_SIG_LEN], s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_bytes(arr)
    }

    pub fn deserialize<'de, D>(d: D) -> Result<[u8; super::ED25519_SIG_LEN], D::Error>
    where
        D: Deserializer<'de>,
    {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = [u8; super::ED25519_SIG_LEN];

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "64-byte signature")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if v.len() != super::ED25519_SIG_LEN {
                    return Err(E::custom(format!(
                        "expected {} bytes; got {}",
                        super::ED25519_SIG_LEN,
                        v.len()
                    )));
                }
                let mut out = [0u8; super::ED25519_SIG_LEN];
                out.copy_from_slice(v);
                Ok(out)
            }
        }
        d.deserialize_bytes(V)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_sign_then_verify_succeeds() {
        let payload = b"E-7 test link payload";
        let sig = placeholder_sign(payload);
        assert!(placeholder_verify(payload, &sig).is_ok());
    }

    #[test]
    fn tampered_payload_fails_verify() {
        let payload = b"E-7 test link payload";
        let sig = placeholder_sign(payload);
        let tampered = b"E-7 test link payloaX";
        assert_eq!(
            placeholder_verify(tampered, &sig),
            Err(CryptoError::SignatureBroken)
        );
    }

    #[test]
    fn canonical_payload_is_stable_128_bytes() {
        let link = Ed25519SignedLink {
            self_blake3: [1u8; 32],
            parent_blake3: [2u8; 32],
            parent_lock_list_blake3: [3u8; 32],
            verifier_set_blake3: [4u8; 32],
            ed25519_sig: [0u8; ED25519_SIG_LEN],
        };
        let payload = canonical_payload(&link);
        assert_eq!(&payload[0..32], &[1u8; 32]);
        assert_eq!(&payload[32..64], &[2u8; 32]);
        assert_eq!(&payload[64..96], &[3u8; 32]);
        assert_eq!(&payload[96..128], &[4u8; 32]);
    }
}
