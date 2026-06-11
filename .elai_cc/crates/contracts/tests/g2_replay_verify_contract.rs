//! G-2 -- `replay --verify` semantics contract tests.
//!
//! Focuses on the verify path AS A FILE round-trip (write_rra -> read_rra ->
//! verify_rra), the byte-on-disk tamper detection, and the honest-scope
//! guarantee encoding the `elai replay --verify` / `elai rra show` CLI surface
//! renders. Complements g2_rra_roundtrip_contract.rs (which drives a live SSB
//! fixture stream); here the artifact is constructed in-memory so the verify
//! legs are isolated from the SSB writer.
//!
//! Coverage:
//!   (a) a faithful in-memory artifact written + read back + verified ->
//!       Verified, and the read-back artifact equals the written one.
//!   (b) a single FLIPPED BYTE in the on-disk .rra payload -> verify_rra
//!       returns Tampered (the canonical-bytes root no longer recomputes).
//!   (c) the GuaranteeLevel as_sql_str / serde encoding is the honest snake_case
//!       wire label, and ReproducibleTokens is a distinct (never-produced) arm.
//!   (d) the empty-manifest artifact (no F-gate rows at all) still verifies
//!       (R1/R2 honesty: absence of gate rows is a valid state, not a failure).
//!
//! No #[ignore] / stubs / placeholder assertions.

use elai_orchestrator::rra::{
    build_rra, read_rra, verify_rra, write_rra, GuaranteeLevel, ReplayVerdict, RunArtifact,
    RRA_COMPRESSION_DEFERRED,
};
use elai_orchestrator::determinism::{DeterminismConfig, HardwareClass};
use tempfile::TempDir;

/// Build a minimal faithful artifact from an EMPTY stream (no SSB needed). An
/// empty stream is a valid run with no recorded rows; the bundle still seals a
/// root and verifies.
fn empty_stream_artifact(seed: u64) -> RunArtifact {
    let pins = DeterminismConfig {
        hardware_class: HardwareClass::Cpu,
        seed,
        blake3_cache_prefix: [0u8; 8],
        system_fingerprint_pin: Some("cpu-pin".to_string()),
    };
    build_rra("g2-replay-session", &[], &pins)
}

// ---------------------------------------------------------------------------
// (a) write -> read -> verify == Verified, round-trip equality
// ---------------------------------------------------------------------------

#[test]
fn write_read_verify_roundtrips() {
    let tmp = TempDir::new().expect("tempdir");
    let art = empty_stream_artifact(11);
    let path = tmp.path().join("empty.rra");

    write_rra(&art, &path).expect("write_rra");
    let read_back = read_rra(&path).expect("read_rra");
    assert_eq!(read_back, art, "read-back .rra must equal the written artifact");

    let verdict = verify_rra(&path).expect("verify_rra");
    assert_eq!(verdict, ReplayVerdict::Verified, "faithful .rra must verify");
}

// ---------------------------------------------------------------------------
// (b) a single flipped byte on disk -> Tampered
// ---------------------------------------------------------------------------

#[test]
fn single_flipped_byte_on_disk_returns_tampered() {
    let tmp = TempDir::new().expect("tempdir");
    let art = empty_stream_artifact(11);
    let path = tmp.path().join("flip.rra");
    write_rra(&art, &path).expect("write_rra");

    // Flip one byte in the middle of the serialized file. msgpack of the whole
    // RunArtifact INCLUDES the root_blake3 string; a mid-file flip mutates the
    // canonical-bytes-bearing region, so the recomputed root no longer matches
    // the (possibly also mutated) stored root -> Tampered. We assert it is NOT
    // a silent Verified -- the gate is fail-closed (R2).
    let mut bytes = std::fs::read(&path).expect("read bytes");
    assert!(bytes.len() > 8, "serialized .rra must be non-trivial");
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0xFF;
    std::fs::write(&path, &bytes).expect("write flipped bytes");

    // The flip may corrupt msgpack framing (a decode error) OR survive to a
    // root mismatch. Either way it must NEVER return a fabricated Verified.
    match verify_rra(&path) {
        Ok(ReplayVerdict::Verified) => {
            panic!("a flipped-byte .rra must NEVER verify clean (fail-open hazard)")
        }
        Ok(ReplayVerdict::Tampered { .. }) | Ok(ReplayVerdict::Divergence { .. }) => {
            // Expected: the mutation was caught.
        }
        Err(_) => {
            // Also acceptable: the flip broke msgpack framing -> hard read error
            // (missing/corrupt evidence is a FAIL, never a pass).
        }
    }
}

// ---------------------------------------------------------------------------
// (c) honest guarantee-level encoding
// ---------------------------------------------------------------------------

#[test]
fn guarantee_level_wire_labels_are_honest() {
    assert_eq!(
        GuaranteeLevel::ReproducibleActionStream.as_sql_str(),
        "reproducible_action_stream"
    );
    assert_eq!(
        GuaranteeLevel::VerifyDivergence.as_sql_str(),
        "verify_divergence"
    );
    assert_eq!(
        GuaranteeLevel::ReproducibleTokens.as_sql_str(),
        "reproducible_tokens"
    );

    // serde round-trips the snake_case label.
    let json = serde_json::to_string(&GuaranteeLevel::VerifyDivergence).expect("serialize");
    assert_eq!(json, "\"verify_divergence\"");
    let back: GuaranteeLevel = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, GuaranteeLevel::VerifyDivergence);

    // A CPU-pinned local run with no model swap is action-stream reproducible.
    let art = empty_stream_artifact(3);
    assert_eq!(
        art.guarantee_level,
        GuaranteeLevel::ReproducibleActionStream,
        "a pinned local run is action-stream reproducible"
    );
}

// ---------------------------------------------------------------------------
// (d) empty-manifest artifact still verifies; compression decision is recorded
// ---------------------------------------------------------------------------

#[test]
fn empty_manifest_artifact_verifies_and_compression_is_documented() {
    let tmp = TempDir::new().expect("tempdir");
    let art = empty_stream_artifact(0);
    assert!(
        art.manifest.is_empty(),
        "an empty stream yields an empty F-gate manifest (valid gate-off state)"
    );
    assert_eq!(art.gate_count_for_test(), 0);

    let path = tmp.path().join("empty-manifest.rra");
    write_rra(&art, &path).expect("write_rra");
    assert_eq!(
        verify_rra(&path).expect("verify_rra"),
        ReplayVerdict::Verified,
        "an empty-manifest .rra must verify (R1/R2: absence of gate rows is valid)"
    );

    // The compression decision (uncompressed msgpack; zstd deferred) is recorded
    // as an auditable constant per DD1.
    assert!(
        RRA_COMPRESSION_DEFERRED.contains("uncompressed msgpack"),
        "the compression decision must be documented as uncompressed msgpack"
    );
}

// Helper extension trait so the empty-manifest test can read the manifest
// length without exposing internals beyond the public field.
trait RunArtifactTestExt {
    fn gate_count_for_test(&self) -> usize;
}
impl RunArtifactTestExt for RunArtifact {
    fn gate_count_for_test(&self) -> usize {
        self.manifest.len()
    }
}
