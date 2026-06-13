//! I-45 — Objective-hash sealed-receipt contract (Layer-4, in-process).
//!
//! Verifies three structural claims from the I-45 WIRING_CONTRACT:
//!
//! (a) seal + verify PASS: an artifact built with a non-empty IdeaFramingStarted
//!     stream carries a non-None objective_hash AND verify_artifact_with_objective
//!     with the SAME text returns Verified.
//!
//! (b) mutated objective: the same artifact verified with DIFFERENT task text
//!     returns ObjectiveHashMismatch (not Verified). THE CRITICAL ANTI-REGRESSION:
//!     if this arm returns Verified the implementation is fail-open.
//!
//! (c) LEGACY receipt: an artifact with objective_hash = None verified with
//!     any (or no) current text returns Verified with an explicit LEGACY path —
//!     never ObjectiveHashMismatch, never ObjectiveUnchecked.
//!
//! All assertions are in-process deterministic; no live sidecar required.
//! Grounding: R2 (missing evidence = hard FAIL); R4 (off-agent artifact);
//! tracing-test rule T-1/T-2 (logs_contain wrapped in assert! where used).

use std::sync::Arc;

use elai_orchestrator::determinism::{DeterminismConfig, HardwareClass, PREFIX_CACHE_KEY_BYTES};
use elai_orchestrator::rra::{
    build_rra, canonical_objective_bytes, verify_artifact, verify_artifact_with_objective,
    ReplayVerdict,
};
use elai_orchestrator::ssb::{
    IdeaFramingStartedPayload, SsbActionType, SsbEntry, SsbPayload,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Minimal DeterminismConfig with a fixed seed (no env reads).
fn test_pins() -> DeterminismConfig {
    DeterminismConfig {
        seed: 42,
        hardware_class: HardwareClass::Cpu,
        system_fingerprint_pin: None,
        blake3_cache_prefix: [0u8; PREFIX_CACHE_KEY_BYTES],
    }
}

/// Build a minimal ordered SSB stream containing ONE IdeaFramingStarted row.
fn stream_with_objective(task_text: &str) -> Vec<SsbEntry> {
    let payload = SsbPayload::IdeaFramingStarted(IdeaFramingStartedPayload {
        session_id: "s-i45-contract".to_owned(),
        prompt: task_text.to_owned(),
        started_at_ms: 1_700_000_000_000,
    });
    let payload_bytes: Arc<[u8]> = rmp_serde::to_vec_named(&payload)
        .expect("msgpack encode")
        .into();
    vec![SsbEntry {
        seq: 1,
        project_id: "test-project".to_owned(),
        session_id: "s-i45-contract".to_owned(),
        agent_id: "cli:run".to_owned(),
        superstep: 0,
        action_type: SsbActionType::IdeaFramingStarted,
        payload: payload_bytes,
        created_at: 1_700_000_000_000,
    }]
}

/// Empty stream -> LEGACY receipt simulation (no IdeaFramingStarted).
fn stream_empty() -> Vec<SsbEntry> {
    vec![]
}

// ---------------------------------------------------------------------------
// (a) seal + verify PASS: objective_hash present, same text -> Verified
// ---------------------------------------------------------------------------

#[test]
fn arm_a_seal_and_verify_pass() {
    let task = "implement the foo feature";
    let stream = stream_with_objective(task);
    let art = build_rra("s-i45-contract", &stream, &test_pins());

    // The artifact must carry a non-None objective hash (the feature was sealed).
    assert!(
        art.objective_hash.is_some(),
        "objective_hash must be Some when IdeaFramingStarted is present in stream"
    );

    // Verify with the SAME task text -> Verified (not ObjectiveUnchecked,
    // not ObjectiveHashMismatch).
    let verdict = verify_artifact_with_objective(&art, Some(task))
        .expect("verify must not error");
    assert_eq!(
        verdict,
        ReplayVerdict::Verified,
        "verify with matching objective text must return Verified, got: {:?}",
        verdict
    );

    // The canonical_objective_bytes of the task must match what was sealed.
    let expected_hash = canonical_objective_bytes(task);
    assert_eq!(
        art.objective_hash.as_deref(),
        Some(expected_hash.as_str()),
        "sealed objective hash must equal canonical_objective_bytes of the task text"
    );
}

// ---------------------------------------------------------------------------
// (b) mutated objective: different text -> ObjectiveHashMismatch (NEVER Verified)
// ---------------------------------------------------------------------------

#[test]
fn arm_b_mutated_objective_fails() {
    let original_task = "implement the foo feature";
    let mutated_task = "implement the BAR feature";

    let stream = stream_with_objective(original_task);
    let art = build_rra("s-i45-contract", &stream, &test_pins());

    // Must have a sealed hash from the original task.
    assert!(
        art.objective_hash.is_some(),
        "precondition: objective_hash present"
    );

    // Verify with a DIFFERENT task text -> ObjectiveHashMismatch.
    // THE CRITICAL ARM: if this returns Verified, the seal is fail-open.
    let verdict = verify_artifact_with_objective(&art, Some(mutated_task))
        .expect("verify must not error");

    match &verdict {
        ReplayVerdict::ObjectiveHashMismatch { sealed_hash, current_hash } => {
            // Confirm the hashes are distinct and match what we'd compute.
            let expected_sealed = canonical_objective_bytes(original_task);
            let expected_current = canonical_objective_bytes(mutated_task);
            assert_eq!(
                sealed_hash, &expected_sealed,
                "sealed_hash in verdict must match hash of original task"
            );
            assert_eq!(
                current_hash, &expected_current,
                "current_hash in verdict must match hash of mutated task"
            );
            assert_ne!(
                sealed_hash, current_hash,
                "sealed and current hashes must differ for a mutated objective"
            );
        }
        other => {
            panic!(
                "mutated objective MUST return ObjectiveHashMismatch, \
                 got {:?} — implementation is fail-open",
                other
            );
        }
    }
}

// ---------------------------------------------------------------------------
// (b2) trailing-whitespace invariance: canonicalization masks trailing spaces/
//      newlines, so a mutation that ONLY appends trailing whitespace hashes
//      identically to the original and verify returns Verified, not Mismatch.
//      This is INTENTIONAL — canonical_objective_bytes calls trim_end() before
//      hashing (rra.rs:302). The test pins this behavior so it is visible and
//      explicit rather than an accidental property of the implementation.
// ---------------------------------------------------------------------------

#[test]
fn arm_b2_trailing_whitespace_is_same_hash() {
    let original_task = "implement the foo feature";
    // Append trailing spaces and a trailing newline — the canonical form strips both.
    let trailing_spaces = "implement the foo feature   ";
    let trailing_newline = "implement the foo feature\n";
    let trailing_mixed = "implement the foo feature \n  ";

    // All four forms must produce the same canonical hash.
    let hash_original = canonical_objective_bytes(original_task);
    let hash_spaces = canonical_objective_bytes(trailing_spaces);
    let hash_newline = canonical_objective_bytes(trailing_newline);
    let hash_mixed = canonical_objective_bytes(trailing_mixed);

    assert_eq!(
        hash_original, hash_spaces,
        "trailing spaces must not change the canonical hash (trim_end invariance, INTENTIONAL)"
    );
    assert_eq!(
        hash_original, hash_newline,
        "trailing newline must not change the canonical hash (trim_end invariance, INTENTIONAL)"
    );
    assert_eq!(
        hash_original, hash_mixed,
        "mixed trailing whitespace must not change the canonical hash (trim_end invariance, INTENTIONAL)"
    );

    // Confirm at the verify level: an artifact sealed with the original text
    // verifies as Verified (not Mismatch) when the caller supplies a trailing-
    // whitespace variant — because canonicalization equalizes them.
    let stream = stream_with_objective(original_task);
    let art = build_rra("s-i45-contract", &stream, &test_pins());

    for variant in &[trailing_spaces, trailing_newline, trailing_mixed] {
        let verdict = verify_artifact_with_objective(&art, Some(variant))
            .expect("verify must not error");
        assert_eq!(
            verdict,
            ReplayVerdict::Verified,
            "trailing-whitespace variant {:?} must verify as Verified (canonical trim_end, INTENTIONAL)",
            variant
        );
    }
}

// ---------------------------------------------------------------------------
// (c) LEGACY receipt (objective_hash = None) -> Verified (explicit LEGACY path)
// ---------------------------------------------------------------------------

#[test]
fn arm_c_legacy_receipt_is_verified() {
    // Build from an empty stream (no IdeaFramingStarted -> objective_hash = None).
    let stream = stream_empty();
    let art = build_rra("s-i45-contract", &stream, &test_pins());

    // Confirm it is truly a LEGACY receipt.
    assert!(
        art.objective_hash.is_none(),
        "precondition: empty stream produces a LEGACY receipt (objective_hash = None)"
    );

    // Verify with no current text -> Verified (LEGACY path, no hash to compare).
    let verdict_no_text = verify_artifact(&art).expect("verify must not error");
    assert_eq!(
        verdict_no_text,
        ReplayVerdict::Verified,
        "LEGACY receipt with no current text must return Verified (no objective hash to check)"
    );

    // Verify with some current text -> also Verified (objective_hash = None, skip check).
    let verdict_with_text = verify_artifact_with_objective(&art, Some("any task text"))
        .expect("verify must not error");
    assert_eq!(
        verdict_with_text,
        ReplayVerdict::Verified,
        "LEGACY receipt with current text must return Verified (no sealed hash to compare against)"
    );

    // Explicitly confirm it is NOT ObjectiveHashMismatch or ObjectiveUnchecked.
    assert!(
        !matches!(
            &verdict_with_text,
            ReplayVerdict::ObjectiveHashMismatch { .. } | ReplayVerdict::ObjectiveUnchecked { .. }
        ),
        "LEGACY receipt must never produce objective-check failure verdicts"
    );
}
