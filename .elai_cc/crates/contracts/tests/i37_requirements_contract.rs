//! I-37 — EARS requirement objects contract tests.
//!
//! Coverage (no #[ignore], no stubs, no placeholder asserts):
//!
//! (a) EARS parsing: all five stereotypes (Ubiquitous / EventDriven / StateDriven /
//!     Optional / UnwantedBehaviour) classify correctly; a non-`shall` line is skipped.
//!
//! (b) verify_requirement: Satisfied (coverage>=floor AND MRs held AND bound test live),
//!     Unverified (no bound tests — shown, not dropped, R1), Regressed (bound test
//!     deleted / below floor / MR failed).
//!
//! (c) content_hash idempotent round-trip: the same normalized text hashes identically;
//!     re-parsing a spec-kit string is idempotent on the set.
//!
//! (d) SSB action_type round-trip: all three requirement_verified_* sql_str values
//!     round-trip via ssb_action_type_from_str_public; schema floor SCHEMA_VERSION_PUBLIC
//!     >= 257 AND SSB_ACTION_TYPES.len() >= 566.
//!
//! (e) REAL-DB production-path SSB row lands: open a TempDir SQLite Ssb, build an
//!     EarsRequirementSet, drive `emit_requirement_verified_to_ssb` (the exact fn the
//!     production `task_runner::drive_task` PLANNING->TESTING_FIRST seam calls), then
//!     query_by_type(RequirementVerified) and assert the row landed + decodes (NOT
//!     struct-only; off-author artifact, wiring-enforcement R4). Includes the Unverified
//!     arm to prove an unverifiable requirement is surfaced, never dropped (R1).
//!
//! Spec references: docs/thefinalglance/I-plan/I-37_ears_requirement_objects.md
//! Reads (mandatory): wiring-enforcement.md R1/R4/R7/R8; CLAUDE.md GRC + ID-5 + blake3-only.

use std::sync::Arc;

use elai_orchestrator::requirements::{
    emit_requirement_verified_to_ssb, parse_ears_line, parse_speckit_str, reverify_set,
    verify_requirement, EarsPattern, EarsRequirement, EarsRequirementSet, VerifiedStatus,
    DEFAULT_MUTATION_COVERAGE_FLOOR,
};
use elai_orchestrator::ssb::{
    decode_ssb_payload, ssb_action_type_from_str_public, RequirementVerifiedKind, SsbActionType,
    SsbPayload, SCHEMA_VERSION_PUBLIC, SSB_ACTION_TYPES,
};

// ---------------------------------------------------------------------------
// (a) EARS parsing — all five stereotypes
// ---------------------------------------------------------------------------

#[test]
fn i37_parses_all_five_ears_patterns() {
    let cases = [
        ("the system shall persist state", EarsPattern::Ubiquitous),
        (
            "when a request arrives the system shall respond",
            EarsPattern::EventDriven,
        ),
        (
            "while charging the system shall show the level",
            EarsPattern::StateDriven,
        ),
        (
            "where premium is on the system shall sync to cloud",
            EarsPattern::Optional,
        ),
        (
            "if the disk is full then the system shall abort the write",
            EarsPattern::UnwantedBehaviour,
        ),
    ];
    for (line, expected) in cases {
        let (pattern, _canonical) =
            parse_ears_line(line).unwrap_or_else(|| panic!("EARS line must parse: {line}"));
        assert_eq!(pattern, expected, "pattern mismatch for: {line}");
    }
    // A line with no EARS modal "shall" is NOT a requirement.
    assert!(
        parse_ears_line("this is just a description").is_none(),
        "a non-shall line must not classify as an EARS requirement"
    );
}

// ---------------------------------------------------------------------------
// (b) verify_requirement — Satisfied / Unverified / Regressed
// ---------------------------------------------------------------------------

#[test]
fn i37_verify_status_transitions() {
    let bound = vec!["test_persist".to_string()];
    let live = vec!["test_persist".to_string()];

    // No bound tests -> Unverified (shown, never dropped: R1).
    assert_eq!(
        verify_requirement(&[], Some(0.99), 0.8, &[], true),
        VerifiedStatus::Unverified,
        "a requirement with no bound tests must be Unverified, never silently passed"
    );

    // Bound test deleted (not in live set) -> Regressed.
    assert_eq!(
        verify_requirement(&bound, Some(0.99), 0.8, &[], true),
        VerifiedStatus::Regressed,
        "a deleted bound test must flip to Regressed"
    );

    // Below floor -> Regressed.
    assert_eq!(
        verify_requirement(&bound, Some(0.50), 0.8, &live, true),
        VerifiedStatus::Regressed,
        "coverage below floor must be Regressed"
    );

    // Failed MR -> Regressed.
    assert_eq!(
        verify_requirement(&bound, Some(0.99), 0.8, &live, false),
        VerifiedStatus::Regressed,
        "a failed metamorphic relation must be Regressed"
    );

    // All conditions met -> Satisfied.
    assert_eq!(
        verify_requirement(&bound, Some(0.90), 0.8, &live, true),
        VerifiedStatus::Satisfied,
        "coverage >= floor AND live bound test AND MR held must be Satisfied"
    );
}

// ---------------------------------------------------------------------------
// (c) content_hash idempotent round-trip
// ---------------------------------------------------------------------------

#[test]
fn i37_content_hash_and_speckit_round_trip_idempotent() {
    let a = EarsRequirement::new("The  System  SHALL  log", EarsPattern::Ubiquitous);
    let b = EarsRequirement::new("the system shall log", EarsPattern::Ubiquitous);
    assert_eq!(
        a.content_hash, b.content_hash,
        "normalized requirement text must hash identically (blake3 drift key)"
    );
    // blake3 produces a 64-hex-char digest.
    assert_eq!(a.content_hash.len(), 64, "content_hash must be a blake3 hex digest");

    let spec = "\
---
# requirements
- the system shall log every error
- when a key is pressed the system shall echo it
the system shall log every error
";
    let first = parse_speckit_str(spec);
    let second = parse_speckit_str(spec);
    assert_eq!(first.len(), 2, "duplicate ubiquitous line must de-dup on content_hash");
    assert_eq!(first, second, "spec-kit parse must be idempotent on the set");
}

// ---------------------------------------------------------------------------
// (d) SSB action_type round-trip + schema floor
// ---------------------------------------------------------------------------

#[test]
fn i37_action_type_round_trips() {
    let arms = [
        ("requirement_verified_satisfied", RequirementVerifiedKind::Satisfied),
        ("requirement_verified_unverified", RequirementVerifiedKind::Unverified),
        ("requirement_verified_regressed", RequirementVerifiedKind::Regressed),
    ];
    for (wire_str, expected_kind) in arms {
        let parsed = ssb_action_type_from_str_public(wire_str)
            .unwrap_or_else(|e| panic!("ssb_action_type_from_str_public({wire_str:?}) failed: {e}"));
        match parsed {
            SsbActionType::RequirementVerified(kind) => {
                assert_eq!(kind, expected_kind, "round-trip kind mismatch for {wire_str:?}");
                assert_eq!(
                    kind.as_sql_str(),
                    wire_str,
                    "as_sql_str() must reproduce the original wire_str for {wire_str:?}"
                );
            }
            other => panic!("expected RequirementVerified for {wire_str:?}; got {other:?}"),
        }
    }
}

#[test]
fn i37_schema_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 257,
        "SCHEMA_VERSION_PUBLIC must be >= 257 (I-37 floor: RequirementVerified added); got {}",
        SCHEMA_VERSION_PUBLIC,
    );
    const I37_TV_FLOOR: usize = 566;
    let total = SSB_ACTION_TYPES.len();
    assert!(
        total >= I37_TV_FLOOR,
        "SSB_ACTION_TYPES.len() must be >= {I37_TV_FLOOR} (I-37 floor); got {total}",
    );
    let has_req = SSB_ACTION_TYPES
        .iter()
        .any(|t| matches!(t, SsbActionType::RequirementVerified(_)));
    assert!(
        has_req,
        "SSB_ACTION_TYPES must include the RequirementVerified representative"
    );
}

// ---------------------------------------------------------------------------
// (e) REAL-DB production-path SSB row lands
// ---------------------------------------------------------------------------

/// Drive the EXACT emit fn the production `task_runner::drive_task`
/// PLANNING->TESTING_FIRST seam calls, against a real TempDir SQLite db, then
/// query the db and assert the RequirementVerified rows landed and decode. This
/// is the off-author artifact (wiring-enforcement R4) — missing rows = FAIL.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i37_real_db_requirement_verified_rows_land() {
    use elai_core::clock::OsClock;
    use elai_orchestrator::ssb::{Ssb, SsbConfig, SsbReader};
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir");
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: dir.path().join("i37-ssb.db"),
                pool_size: Some(2),
                project_id: "i37-contract".into(),
                session_id: "i37-e".into(),
            },
            clock,
        )
        .expect("ssb open"),
    );

    // Build a set with one Satisfied requirement (bound test live, coverage>=floor)
    // and one Unverified requirement (no bound tests — must be SHOWN, not dropped).
    let mut set = EarsRequirementSet::new();

    let mut satisfied = EarsRequirement::new(
        "the system shall persist state",
        EarsPattern::Ubiquitous,
    );
    satisfied.bound_tests.push("test_persist_state".to_string());
    satisfied.mutation_coverage = Some(0.95);
    set.insert(satisfied);

    let unverified = EarsRequirement::new(
        "when a timer fires the system shall flush the buffer",
        EarsPattern::EventDriven,
    );
    set.insert(unverified);

    // Re-verify exactly as the production seam does (DEFAULT floor, live test set,
    // empty held-MR set -> vacuously held for records with no MRs).
    reverify_set(
        &mut set,
        DEFAULT_MUTATION_COVERAGE_FLOOR,
        &["test_persist_state".to_string()],
        &[],
    );

    // The Satisfied requirement is Satisfied; the no-bound-test one is Unverified.
    assert_eq!(set.requirements[0].verified_status, VerifiedStatus::Satisfied);
    assert_eq!(set.requirements[1].verified_status, VerifiedStatus::Unverified);

    // Drive the production emit fn.
    emit_requirement_verified_to_ssb(&ssb, "i37-contract-agent", 1, &set).await;

    // Off-author artifact (R4): the Satisfied row must have landed.
    let satisfied_rows = ssb
        .query_by_type(SsbActionType::RequirementVerified(
            RequirementVerifiedKind::Satisfied,
        ))
        .expect("query_by_type Satisfied");
    assert!(
        !satisfied_rows.is_empty(),
        "expected >=1 RequirementVerified(Satisfied) row; found 0 (R1: missing evidence = FAIL)"
    );

    // R1: the Unverified requirement must ALSO be in the db (shown, never dropped).
    let unverified_rows = ssb
        .query_by_type(SsbActionType::RequirementVerified(
            RequirementVerifiedKind::Unverified,
        ))
        .expect("query_by_type Unverified");
    assert!(
        !unverified_rows.is_empty(),
        "expected >=1 RequirementVerified(Unverified) row; the unverifiable requirement \
         must be SHOWN in the cockpit, never silently dropped (R1)"
    );

    // Decode the Satisfied row and verify field content.
    let decoded = decode_ssb_payload(&satisfied_rows[0].payload).expect("decode_ssb_payload");
    match decoded {
        SsbPayload::RequirementVerified(p) => {
            assert_eq!(p.kind, RequirementVerifiedKind::Satisfied, "decoded kind");
            assert_eq!(p.pattern, "ubiquitous", "decoded pattern label");
            assert_eq!(p.bound_test_count, 1, "decoded bound_test_count");
            assert_eq!(p.mutation_coverage, Some(0.95), "decoded mutation_coverage");
            assert_eq!(p.content_hash.len(), 64, "content_hash must be a blake3 hex digest");
        }
        other => panic!("expected SsbPayload::RequirementVerified; got {other:?}"),
    }
}
