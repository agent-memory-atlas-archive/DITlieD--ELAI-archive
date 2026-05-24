//! C-36 contract: MuCoCo Mutation Operator Catalog.
//!
//! Pins:
//!   * SCHEMA_VERSION_PUBLIC >= 104 (atomic 1-variant lockstep bump for the
//!     C-36 MuCoCo catalog landing — see ssb.rs header comment at L1044).
//!   * `SsbActionType::MucocoMutationApplied` round-trips via
//!     `ssb_action_type_from_str_public` / `SsbActionType::as_sql_str`.
//!   * `MucocoMutationAppliedPayload` shape (operator, language, source_hash,
//!     applied, skip_reason, timestamp_ms) and serde round-trip preserve a
//!     non-None skip_reason on applied=False (paper sec S3.p3.1
//!     applicability-first contract).
//!   * `list_mucoco_operators()` enumerates exactly 11 paper-grounded
//!     operators (Table 9 / Appendix B of arXiv:2604.19086).
//!   * Each `MucocoOperator` variant round-trips through `as_str` / `FromStr`.
//!   * `MucocoError::UnknownOperator` is the rejection path for unknown
//!     operator names (no panics).
//!
//! Paper: arXiv:2604.19086, interface map at
//! workflow/papers/mucoco-2604.19086-interface.md (HIGH reliability).
//! Per CLAUDE.md feedback_no_cargo_test_unless_permitted, this file is
//! authored but NOT executed — use cargo check -p elai-orchestrator only.

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, MucocoMutationAppliedPayload, SsbActionType,
    SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::{list_mucoco_operators, MucocoError, MucocoOperator};

// ---------------------------------------------------------------------------
// SSB schema + variant pins
// ---------------------------------------------------------------------------

#[test]
fn c36_schema_version_public_floor_is_104() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 104,
        "C-36 atomic 1-variant lockstep bump: floor is 104 (see \
         ssb.rs:1043 + workflow/papers/mucoco-2604.19086-interface.md). \
         Current: {SCHEMA_VERSION_PUBLIC}",
    );
}

#[test]
fn c36_mucoco_action_type_round_trips_through_str() {
    let variant = SsbActionType::MucocoMutationApplied;
    assert_eq!(variant.as_sql_str(), "mucoco_mutation_applied");
    let parsed = ssb_action_type_from_str_public("mucoco_mutation_applied")
        .unwrap_or_else(|e| panic!("from_str must accept 'mucoco_mutation_applied': {e}"));
    assert_eq!(parsed, variant);
}

#[test]
fn c36_mucoco_payload_applied_path_serde_roundtrip() {
    let p = MucocoMutationAppliedPayload {
        operator: "demorgan".to_string(),
        language: "python".to_string(),
        source_hash: 0xdead_beef_cafe_babe,
        applied: true,
        skip_reason: None,
        timestamp_ms: 1_700_000_000_000,
    };
    let bytes = serde_json::to_vec(&p).expect("serialize");
    let back: MucocoMutationAppliedPayload =
        serde_json::from_slice(&bytes).expect("deserialize");
    assert_eq!(back, p);
    assert!(back.applied);
    assert!(back.skip_reason.is_none());
}

#[test]
fn c36_mucoco_payload_skipped_path_preserves_non_empty_reason() {
    // Paper sec S3.p3.1: applicability check first; applied=False MUST
    // carry a non-empty skip_reason (stub_ban contract).
    let p = MucocoMutationAppliedPayload {
        operator: "for_to_while".to_string(),
        language: "python".to_string(),
        source_hash: 0x0011_2233_4455_6677,
        applied: false,
        skip_reason: Some("no for-loop without else-clause found".to_string()),
        timestamp_ms: 1_700_000_001_000,
    };
    let bytes = serde_json::to_vec(&p).expect("serialize");
    let back: MucocoMutationAppliedPayload =
        serde_json::from_slice(&bytes).expect("deserialize");
    assert_eq!(back, p);
    assert!(!back.applied);
    let reason = back.skip_reason.expect("skip_reason MUST be present on applied=False");
    assert!(
        !reason.is_empty(),
        "MucocoMutationAppliedPayload.skip_reason MUST be non-empty (stub_ban / paper sec S3.p3.1)"
    );
}

// ---------------------------------------------------------------------------
// Operator registry + round-trip pins
// ---------------------------------------------------------------------------

#[test]
fn c36_list_mucoco_operators_has_eleven_paper_grounded_variants() {
    let ops = list_mucoco_operators();
    assert_eq!(
        ops.len(),
        11,
        "MuCoCo paper Table 9 / Appendix B defines exactly 11 operators; got {}",
        ops.len()
    );
}

#[test]
fn c36_list_mucoco_operators_includes_every_paper_named_operator() {
    let ops = list_mucoco_operators();
    let names: Vec<&'static str> = ops.iter().map(|o| o.as_str()).collect();
    let expected = [
        // Lexical (paper B.0.1)
        "random_mutation",
        "sequential_mutation",
        "literal_format",
        // Syntactic (paper B.0.2)
        "for_to_enumerate",
        "for_to_while",
        // Logical (paper B.0.3)
        "demorgan",
        "boolean_literal",
        "commutative_reorder",
        "constant_unfolding",
        "constant_unfolding_add",
        "constant_unfolding_mul",
    ];
    for name in expected.iter() {
        assert!(
            names.contains(name),
            "MuCoCo operator {name:?} missing from registry; got {names:?}"
        );
    }
    assert_eq!(names.len(), expected.len(), "registry size mismatch");
}

#[test]
fn c36_every_mucoco_operator_round_trips_as_str_to_from_str() {
    for op in list_mucoco_operators() {
        let s = op.as_str();
        let parsed: MucocoOperator = s
            .parse()
            .unwrap_or_else(|e| panic!("operator {s} failed to round-trip from_str: {e}"));
        assert_eq!(parsed, op, "round-trip mismatch for {s}");
    }
}

#[test]
fn c36_mucoco_operator_from_str_rejects_unknown_name() {
    let r: Result<MucocoOperator, _> = "this_is_not_a_real_operator".parse();
    match r {
        Err(MucocoError::UnknownOperator(name)) => {
            assert_eq!(name, "this_is_not_a_real_operator");
        }
        _ => panic!("expected MucocoError::UnknownOperator for unknown name"),
    }
}

#[test]
fn c36_mucoco_operator_display_matches_as_str() {
    for op in list_mucoco_operators() {
        assert_eq!(
            format!("{op}"),
            op.as_str(),
            "Display must equal as_str for snake_case wire stability"
        );
    }
}

#[test]
fn c36_mucoco_operator_variants_have_unique_wire_tags() {
    // Defensive: as_sql_str-style 1:1 invariant. Two operators must not
    // map to the same snake_case wire tag (would silently collapse SSB
    // analytics and break the from_str round-trip).
    use std::collections::HashSet;
    let mut seen: HashSet<&'static str> = HashSet::new();
    for op in list_mucoco_operators() {
        let s = op.as_str();
        assert!(
            seen.insert(s),
            "duplicate MuCoCo wire tag {s:?} produced by MucocoOperator::as_str"
        );
    }
}
