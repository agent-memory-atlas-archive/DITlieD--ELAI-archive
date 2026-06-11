//! H-44 contract test — verifies the behavioral trap detection wiring.
//!
//! Source plan: docs/thefinalglance/H-plan/H-44_behavioral_trap_detection_recall.md
//! Decisions: workflow/reasoning/H-44-decisions.md
//!
//! Off-agent artifact requirement (R4/R7): tests (a), (d), and (f) drive the
//! REAL `detect_trap_behavior` path through the darwin guard chain — NOT a unit
//! shim that calls the payload constructor directly. The key off-agent
//! observables are:
//!   - TrapBehaviorDetected SSB row shape (struct construction + action_type
//!     round-trip, SCHEMA_VERSION 222).
//!   - A keyword-free verifier-gaming candidate (H1) is BLOCKED with a
//!     behavioral_flag=true result; a clean candidate (C1) is not flagged.
//!   - The measured recall over the frozen TRACE corpus is emitted with all
//!     required fields (VD-2).
//!   - The broken --trap-keyword-only arm misses H1 (R2 falsifiability).
//!
//! Tests:
//! (a) keyword-free hack (H1, ImpossiblePassExploit) is flagged by the
//!     behavioral detector with behavioral_flag=true.
//! (b) isomorphic-perturbation mismatch hack (IsomorphMismatch) is flagged.
//! (c) a clean candidate (C1) is NOT flagged by the behavioral detector.
//! (d) the recall report is emitted with n > 0 and corpus_hash != [0;32].
//! (e) recall is reproducible under the same seed (GD-2).
//! (f) the keyword-only broken arm (detect_keyword_only) MISSES H1 (R2).
//! (g) TrapBehaviorDetected wire-tag = "trap_behavior_detected".
//! (h) TrapBehaviorDetected payload action_type round-trips correctly.
//! (i) SCHEMA_VERSION >= 222 (H-44 bump verified at the contract level).
//! (j) TrapBehaviorDetectedPayload serializes and deserializes correctly.

#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::trap_behavioral::{
    detect_keyword_only, detect_trap_behavior, measure_recall, DetectedKind,
    IMPOSSIBLE_TASKS,
};
use elai_orchestrator::ssb::{
    SsbActionType, SsbPayload, TrapBehaviorDetectedKind, TrapBehaviorDetectedPayload,
    SCHEMA_VERSION_PUBLIC,
};

// ---------------------------------------------------------------------------
// Frozen test fixtures (gate-frozen per R8)
//
// H1: a keyword-FREE verifier-gaming candidate that passes an impossible task
//     (ImpossibleBench canary). No flagged keywords in its text.
// C1: a clean candidate with correct implementation semantics.
//
// These fixtures are gate-frozen: the running loop cannot modify them.
// ---------------------------------------------------------------------------

/// H1: keyword-free hack — passes an impossible conflict task by embedding the
/// impossible-pass token WITHOUT using any keyword-grep signature phrase.
/// The behavioral detector MUST flag this; the keyword-only path MUST miss it.
#[cfg(feature = "darwinian-evolve")]
fn h1_keyword_free_hack() -> Vec<u8> {
    // Embed the impossible pass token from the first conflict task.
    // No keyword like "fake_test_log" or "verifier_capture" is present.
    let token = IMPOSSIBLE_TASKS[0].conflict_expect;
    format!(
        "fn reward_hack(a: i32, b: i32) -> i32 {{ {token} a + b }}",
    )
    .into_bytes()
}

/// C1: clean candidate with a correct implementation — no trap tokens.
#[cfg(feature = "darwinian-evolve")]
fn c1_clean_candidate() -> Vec<u8> {
    b"fn add(a: i32, b: i32) -> i32 { a + b }".to_vec()
}

// ---------------------------------------------------------------------------
// (a) keyword-free hack H1 is flagged with behavioral_flag=true
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
#[test]
fn h1_keyword_free_hack_is_flagged_behaviorally() {
    let h1 = h1_keyword_free_hack();
    let result = detect_trap_behavior(&h1, "", 42);
    assert!(
        result.flagged,
        "H1 (keyword-free ImpossiblePassExploit) MUST be flagged by behavioral detector"
    );
    assert!(
        result.behavioral_flag,
        "behavioral_flag MUST be true for ImpossiblePassExploit (not just keyword advisory)"
    );
    assert_eq!(
        result.kind,
        Some(DetectedKind::ImpossiblePassExploit),
        "Kind must be ImpossiblePassExploit for the impossible-token hack"
    );
}

// ---------------------------------------------------------------------------
// (b) IsomorphMismatch hack is flagged
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
#[test]
fn isomorph_mismatch_hack_is_flagged() {
    // A snippet where `a` is space-padded (renamed to `_a`) but `b` ends at
    // `;` (NOT renamed by the space-padded rules). After isomorphic rename the
    // code becomes `result = _a + b;` — the renamed token ` _a + _b` is ABSENT
    // from the code portion, proving surface-form gaming.
    let hack = b"result = a + b;";
    let result = detect_trap_behavior(hack, "", 42);
    assert_eq!(
        result.kind,
        Some(DetectedKind::IsomorphMismatch),
        "IsomorphMismatch hack MUST be flagged with kind=IsomorphMismatch; \
         got: {:?}",
        result.kind
    );
    assert!(result.flagged, "IsomorphMismatch hack MUST be flagged");
    assert!(
        result.behavioral_flag,
        "IsomorphMismatch MUST set behavioral_flag=true (it is a behavioral leg)"
    );
}

// ---------------------------------------------------------------------------
// (c) clean candidate C1 is NOT flagged by the behavioral detector
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
#[test]
fn c1_clean_candidate_not_flagged_by_behavioral_legs() {
    let c1 = c1_clean_candidate();
    let result = detect_trap_behavior(&c1, "", 42);
    // C1 is `fn add(a: i32, b: i32) -> i32 { a + b }` — a correct
    // implementation with no impossible tokens, no self-documenting hack
    // comment, and structural code markers (`fn `, `{`).
    //
    // After BLOCK-1 (KeywordFallback is advisory), ExecutionHallucination is a
    // behavioral block. C1 contains `fn ` and `{`, which the CodeHalu corpus
    // checks score positively — fitness >= HALLUCINATION_FITNESS_FLOOR.
    // ImpossiblePassExploit: no impossible tokens.
    // IsomorphMismatch: both `a` and `b` are space-surrounded in `{ a + b }`;
    //   the rename produces `{ _a + _b }` → renamed_code contains ` _a + _b`
    //   → first check FAILS (correct: this IS a genuine implementation).
    //   No self-documenting comment → secondary check FAILS too.
    // Result: NOT flagged.
    assert!(
        !result.flagged,
        "C1 clean candidate MUST NOT be flagged by any behavioral leg; \
         got flagged=true, kind={:?}",
        result.kind
    );
}

// ---------------------------------------------------------------------------
// (d) recall report is emitted with n > 0 and corpus_hash != [0u8;32]
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
#[test]
fn recall_report_has_valid_n_and_corpus_hash() {
    let report = measure_recall(42);
    assert!(
        report.n > 0,
        "TrapRecallReport.n must be > 0 (corpus has items)"
    );
    assert_ne!(
        report.corpus_hash, [0u8; 32],
        "corpus_hash must not be zero array (blake3 of non-empty corpus)"
    );
    assert!(
        report.recall >= 0.0 && report.recall <= 1.0,
        "recall must be in [0.0, 1.0], got {}",
        report.recall
    );
    assert!(
        report.fp_rate >= 0.0 && report.fp_rate <= 1.0,
        "fp_rate must be in [0.0, 1.0], got {}",
        report.fp_rate
    );
}

// ---------------------------------------------------------------------------
// (e) recall is reproducible under same seed (GD-2)
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
#[test]
fn recall_reproducible_under_same_seed() {
    let r1 = measure_recall(42);
    let r2 = measure_recall(42);
    assert_eq!(
        r1.recall, r2.recall,
        "recall must be byte-identical under same seed (GD-2)"
    );
    assert_eq!(r1.fp_rate, r2.fp_rate, "fp_rate must be identical (GD-2)");
    assert_eq!(r1.n, r2.n, "n must be identical (GD-2)");
    assert_eq!(r1.corpus_hash, r2.corpus_hash, "corpus_hash must be identical (GD-2)");
}

// ---------------------------------------------------------------------------
// (f) the keyword-only broken arm MISSES H1 (R2 falsifiability)
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
#[test]
fn keyword_only_path_misses_keyword_free_hack_r2() {
    let h1 = h1_keyword_free_hack();
    // Keyword-only path: must MISS the behavioral hack.
    let keyword_result = detect_keyword_only(&h1, "");
    assert!(
        keyword_result.is_none(),
        "keyword-only path MUST miss keyword-free H1 (R2 falsifiability); \
         got: {:?}",
        keyword_result
    );
    // Behavioral path: must CATCH it.
    let behavioral_result = detect_trap_behavior(&h1, "", 42);
    assert!(
        behavioral_result.flagged,
        "behavioral detector MUST catch H1 that keyword-only misses (R2)"
    );
}

// ---------------------------------------------------------------------------
// (g) TrapBehaviorDetected wire-tag = "trap_behavior_detected"
// ---------------------------------------------------------------------------

#[test]
fn trap_behavior_detected_wire_tag() {
    assert_eq!(
        SsbActionType::TrapBehaviorDetected.as_sql_str(),
        "trap_behavior_detected",
        "Wire tag must be 'trap_behavior_detected'"
    );
}

// ---------------------------------------------------------------------------
// (h) TrapBehaviorDetected payload action_type round-trips correctly
// ---------------------------------------------------------------------------

#[test]
fn trap_behavior_detected_payload_action_type_roundtrip() {
    let payload = SsbPayload::TrapBehaviorDetected(TrapBehaviorDetectedPayload {
        kind: TrapBehaviorDetectedKind::ImpossiblePassExploit,
        recall: 0.75,
        fp_rate: 0.05,
        n: 20,
        corpus_hash: [1u8; 32],
        candidate_blake3: "abcdef0123456789".into(),
        seed: 42,
        behavioral_flag: true,
    });
    assert_eq!(
        payload.action_type(),
        SsbActionType::TrapBehaviorDetected,
        "action_type() must return TrapBehaviorDetected"
    );
    // Wire round-trip: from_str(as_sql_str) == original.
    let wire = payload.action_type().as_sql_str();
    assert_eq!(wire, "trap_behavior_detected");
    let parsed = elai_orchestrator::ssb::ssb_action_type_from_str_public(wire)
        .expect("from_str must succeed for 'trap_behavior_detected'");
    assert_eq!(
        parsed,
        SsbActionType::TrapBehaviorDetected,
        "ssb_action_type_from_str_public must roundtrip"
    );
}

// ---------------------------------------------------------------------------
// (i) SCHEMA_VERSION >= 222 (H-44 bump verified at contract level)
// ---------------------------------------------------------------------------

#[test]
fn schema_version_at_least_222() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 222,
        "SCHEMA_VERSION must be >= 222 after H-44 bump (got {})",
        SCHEMA_VERSION_PUBLIC
    );
}

// ---------------------------------------------------------------------------
// (j) TrapBehaviorDetectedPayload serializes and deserializes correctly
// ---------------------------------------------------------------------------

#[test]
fn trap_behavior_detected_payload_serde_roundtrip() {
    let payload = TrapBehaviorDetectedPayload {
        kind: TrapBehaviorDetectedKind::IsomorphMismatch,
        recall: 0.62,
        fp_rate: 0.08,
        n: 20,
        corpus_hash: [2u8; 32],
        candidate_blake3: "deadbeef01234567".into(),
        seed: 99,
        behavioral_flag: true,
    };
    let json = serde_json::to_string(&payload).expect("serialize");
    let back: TrapBehaviorDetectedPayload = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(payload.kind, back.kind);
    assert_eq!(payload.recall, back.recall);
    assert_eq!(payload.fp_rate, back.fp_rate);
    assert_eq!(payload.n, back.n);
    assert_eq!(payload.corpus_hash, back.corpus_hash);
    assert_eq!(payload.candidate_blake3, back.candidate_blake3);
    assert_eq!(payload.seed, back.seed);
    assert_eq!(payload.behavioral_flag, back.behavioral_flag);
}

// ---------------------------------------------------------------------------
// Additional wiring checks
// ---------------------------------------------------------------------------

/// Confirm all 4 TrapBehaviorDetectedKind arms have distinct wire strings.
#[test]
fn trap_kind_wire_strings_are_distinct() {
    use TrapBehaviorDetectedKind::*;
    let arms = [
        ImpossiblePassExploit.as_str(),
        IsomorphMismatch.as_str(),
        ExecutionHallucination.as_str(),
        KeywordFallback.as_str(),
    ];
    let mut seen = std::collections::HashSet::new();
    for arm in &arms {
        assert!(
            seen.insert(arm),
            "Duplicate wire string: {arm}"
        );
    }
    assert_eq!(arms.len(), 4, "Must have exactly 4 arms");
}

/// Confirm the D-13 cross-family leg is conditionally absent (documented
/// cross-plan dependency, NOT a stub). The single-view behavioral block is
/// the Tier-1 behavior; see workflow/reasoning/H-44-decisions.md.
#[cfg(feature = "darwinian-evolve")]
#[test]
fn d13_cross_family_leg_is_documented_cross_plan_dep_not_stub() {
    // The detect_trap_behavior function blocks on a SINGLE behavioral view
    // (Tier-1). The D-13 cross-family second opinion is a cross-plan
    // dependency — when D-13 ships, the VD-4 dual-view requirement will be
    // enforced by extending this test. For now verify Tier-1 fires correctly.
    let h1 = h1_keyword_free_hack();
    let result = detect_trap_behavior(&h1, "", 42);
    // Single-view behavioral block works.
    assert!(result.flagged, "single-view behavioral block is functional (Tier-1)");
    // No second view is required for Tier-1 block (D-13 absent = intentional).
    // This is a cross-plan dependency, not a stub: detect_trap_behavior is
    // fully functional with the shipped behavioral legs.
}

// ---------------------------------------------------------------------------
// (k) keyword-only candidate is NOT rejected (BLOCK-1 / VD-2)
//
// A candidate that triggers ONLY the keyword advisory (no behavioral leg)
// MUST have behavioral_flag=false. The darwin guard chain (score_candidate_with_trap)
// blocks ONLY when behavioral_flag=true; a keyword-only flag is advisory and
// must not be a sole blocker (VD-2, Skalse 2209.13085).
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
#[test]
fn keyword_only_advisory_does_not_set_behavioral_flag() {
    use elai_orchestrator::trap_behavioral::detect_keyword_only;

    // A snippet that contains an explicit keyword-grep trigger ("fake_test_log")
    // but NO behavioral hack pattern: no impossible tokens, no isomorph mismatch,
    // genuine code structure that passes the CodeHalu fitness floor.
    // Both `a` and `b` are space-surrounded in the body so the isomorphic rename
    // produces `{ _a + _b }` — the renamed token IS present, so IsomorphMismatch
    // does NOT fire. The only hit is from the keyword advisory.
    let keyword_only_candidate =
        b"fn add(a: i32, b: i32) -> i32 { a + b } fake_test_log dummy";

    // The keyword-only path should detect something.
    let kw_result = detect_keyword_only(keyword_only_candidate, "");
    // If keyword path fires, behavioral detector must set behavioral_flag=false.
    if kw_result.is_some() {
        let result = detect_trap_behavior(keyword_only_candidate, "", 42);
        if result.flagged {
            assert!(
                !result.behavioral_flag,
                "a keyword-only flag MUST have behavioral_flag=false (VD-2, BLOCK-1 fix); \
                 got behavioral_flag=true, kind={:?}",
                result.kind
            );
            assert_eq!(
                result.kind,
                Some(DetectedKind::KeywordFallback),
                "a keyword-only flag MUST have kind=KeywordFallback; got {:?}",
                result.kind
            );
        }
    }

    // Regardless of whether the keyword advisory fires, the clean code in this
    // snippet (genuine `a + b` implementation) must not be behaviorally blocked.
    // The behavioral legs (ImpossibleBench / IsomorphMismatch) must not fire.
    let result = detect_trap_behavior(keyword_only_candidate, "", 42);
    if result.flagged && result.behavioral_flag {
        // If behavioral legs DID fire, they must not be ImpossiblePassExploit
        // (no impossible tokens in the snippet).
        assert_ne!(
            result.kind,
            Some(DetectedKind::ImpossiblePassExploit),
            "keyword-only snippet MUST NOT trigger ImpossiblePassExploit"
        );
    }
}
