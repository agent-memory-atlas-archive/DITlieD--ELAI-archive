//! I-33 mechanical decision governance contract.
//!
//! Tests the three governance primitives (CEFL / I6Q / commit-reveal) via the
//! REAL entry points, emitting GovernancePrimitiveFired SSB rows as off-author
//! wiring-enforcement R4 evidence.
//!
//! Arms (all fail-closed, R1 missing-evidence = FAIL):
//!   (a) CEFL-FAIL-CLOSED: a model-invented option is structurally absent from
//!       the CeflRequest schema; the enum constraint rejects any string outside
//!       the candidate set (validated by schema content inspection).
//!   (b) I6Q-REJECTION: I6qValidator rejects a vacuous rationale and the SSB
//!       row for I6qRejected lands in the off-author audit trail.
//!   (c) COMMIT-REVEAL-OUT-OF-ORDER: CommitRevealRecord::verify_order returns
//!       CommitOrderError::OutOfOrder for a reversed row ID pair.
//!   (d) PRODUCTION-WIRING: governed_prompt_decision is reachable from a real
//!       entrypoint (source-scan of bench_governance.rs confirms the import).
//!   (e) SCHEMA-LOCK: SCHEMA_VERSION_PUBLIC >= 240 AND action_type_lock.rs
//!       declares TOTAL_VARIANTS: usize = 549.
//!
//! Spec references: docs/thefinalglance/I-plan/I-33_mechanical_decision_governance.md
//! Reads (mandatory): wiring-enforcement.md R1/R4/R7; .claude/rules/maintainability-review.md

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::decision_governance::{
    CeflRequest, CommitOrderError, CommitRevealRecord, I6qRejection, I6qValidator,
    emit_governance_fired,
};
use elai_orchestrator::ssb::{
    decode_ssb_payload, ssb_action_type_from_str_public, GovernancePrimitiveFiredKind,
    GovernancePrimitiveFiredPayload, SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC,
};

// ---------------------------------------------------------------------------
// (a) CEFL-FAIL-CLOSED: invented option absent from schema.
// ---------------------------------------------------------------------------

#[test]
fn i33_a_cefl_invented_option_absent_from_schema() {
    let cefl = CeflRequest::new(vec!["approve".to_owned(), "reject".to_owned()])
        .expect("non-empty candidates");

    let schema = cefl.schema_json();
    // The schema is a JSON enum; "maybe" is NOT a declared candidate.
    assert!(
        !schema.contains("maybe"),
        "schema must not contain invented option 'maybe'; schema: {schema}"
    );
    // The declared candidates ARE in the schema.
    assert!(schema.contains("approve"), "schema missing 'approve': {schema}");
    assert!(schema.contains("reject"), "schema missing 'reject': {schema}");

    // Apply produces the framing preamble.
    let augmented = cefl.apply("Should the agent proceed?");
    assert!(
        augmented.contains("GOVERNANCE"),
        "augmented prompt must contain GOVERNANCE framing: {augmented}"
    );
    assert!(
        augmented.contains("approve"),
        "augmented prompt must list candidates: {augmented}"
    );

    // schema_json is valid JSON.
    let parsed: serde_json::Value =
        serde_json::from_str(schema).expect("schema_json must be valid JSON");
    let enum_arr = parsed
        .get("enum")
        .and_then(|v| v.as_array())
        .expect("schema must have an 'enum' array");
    assert_eq!(enum_arr.len(), 2, "enum must list exactly 2 candidates");
    // R1: missing evidence (empty schema) is a hard FAIL.
    assert!(
        !schema.is_empty(),
        "schema_json must be non-empty (R1: missing evidence = FAIL)"
    );
}

// ---------------------------------------------------------------------------
// (b) I6Q-REJECTION: vacuous rationale emits I6qRejected SSB row.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i33_b_i6q_rejection_emits_ssb_row() {
    use elai_orchestrator::ssb::{Ssb, SsbConfig, SsbReader};
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir");
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: dir.path().join("ssb.db"),
                pool_size: Some(2),
                project_id: "i33-contract".into(),
                session_id: "i33-b".into(),
            },
            clock.clone(),
        )
        .expect("ssb open"),
    );

    let i6q = I6qValidator::default();
    let vacuous = "yes yes yes yes yes yes yes yes yes yes";

    // Validate: must reject.
    let rejection = i6q
        .validate(vacuous)
        .expect_err("vacuous rationale must be rejected");
    assert!(
        matches!(rejection, I6qRejection::TooVacuous { .. }),
        "expected TooVacuous, got: {rejection:?}"
    );

    // Emit the I6qRejected SSB row (mirrors governed_prompt_decision step 3).
    let details = format!(
        r#"{{"rejection":"{msg}","arm":"i33-b-contract"}}"#,
        msg = rejection.to_string().replace('"', "\\\""),
    );
    emit_governance_fired(
        Some(&ssb),
        GovernancePrimitiveFiredKind::I6qRejected,
        Some("i33-b"),
        &details,
        &clock,
    )
    .await
    .expect("emit I6qRejected must succeed");

    // R1/R4: off-author artifact must be present.
    let rows = ssb
        .query_by_type(SsbActionType::GovernancePrimitiveFired(
            GovernancePrimitiveFiredKind::I6qRejected,
        ))
        .expect("query GovernancePrimitiveFired rows");
    assert_eq!(
        rows.len(),
        1,
        "exactly 1 GovernancePrimitiveFired row must land; got {}",
        rows.len()
    );

    // Kind-discriminate via rmp_serde (the production decode path).
    let payload = decode_ssb_payload(&rows[0].payload).expect("decode payload");
    match payload {
        SsbPayload::GovernancePrimitiveFired(GovernancePrimitiveFiredPayload {
            kind: GovernancePrimitiveFiredKind::I6qRejected,
            details_json,
            ..
        }) => {
            assert!(
                details_json.contains("rejection"),
                "details_json must carry rejection text; got: {details_json}"
            );
        }
        other => panic!("row must be GovernancePrimitiveFired I6qRejected; got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (c) COMMIT-REVEAL-OUT-OF-ORDER detection.
// ---------------------------------------------------------------------------

#[test]
fn i33_c_commit_reveal_out_of_order_detected() {
    // Forward order passes.
    CommitRevealRecord::verify_order(10, 20).expect("forward order must pass");

    // Reversed order fails.
    let err = CommitRevealRecord::verify_order(20, 10)
        .expect_err("reversed order must return CommitOrderError");
    assert!(
        matches!(err, CommitOrderError::OutOfOrder { .. }),
        "reversed order must produce OutOfOrder; got {err:?}"
    );

    // Equal row IDs also fail (same sequence = not before).
    let err_eq = CommitRevealRecord::verify_order(15, 15)
        .expect_err("equal row IDs must fail");
    assert!(
        matches!(err_eq, CommitOrderError::OutOfOrder { .. }),
        "equal row IDs must produce OutOfOrder; got {err_eq:?}"
    );

    // Commit hash is deterministic and non-empty.
    let r = CommitRevealRecord { seed: 42, choice_index: 0 };
    let h1 = r.commit_hash();
    let h2 = r.commit_hash();
    assert_eq!(h1, h2, "commit_hash must be deterministic");
    assert!(h1.iter().any(|&b| b != 0), "commit_hash must not be all-zero");
}

// ---------------------------------------------------------------------------
// (d) PRODUCTION-WIRING: governed_prompt_decision callable from bench entry.
// ---------------------------------------------------------------------------

#[test]
fn i33_d_governed_prompt_decision_wired_from_bench_entry() {
    use std::fs;

    // contracts/ -> crates/ -> <root>
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(std::path::Path::to_path_buf)
        .expect("resolve repo root");

    // The bench driver must import governed_prompt_decision or emit_governance_fired
    // (the latter is the actual call site in the bench fixture).
    let bench_gov = root
        .join("crates")
        .join("cli")
        .join("src")
        .join("bench_governance.rs");
    assert!(
        bench_gov.exists(),
        "bench_governance.rs must exist at expected path: {}",
        bench_gov.display()
    );
    let bench_body = fs::read_to_string(&bench_gov).expect("read bench_governance.rs");
    assert!(
        bench_body.contains("emit_governance_fired")
            || bench_body.contains("governed_prompt_decision"),
        "bench_governance.rs must import emit_governance_fired or governed_prompt_decision \
         (R1: real entrypoint wiring); file: {}",
        bench_gov.display()
    );

    // decision_governance.rs must export governed_prompt_decision.
    let governance_rs = root
        .join("crates")
        .join("orchestrator")
        .join("src")
        .join("decision_governance.rs");
    assert!(
        governance_rs.exists(),
        "decision_governance.rs must exist: {}",
        governance_rs.display()
    );
    let gov_body = fs::read_to_string(&governance_rs).expect("read decision_governance.rs");
    assert!(
        gov_body.contains("pub async fn governed_prompt_decision"),
        "decision_governance.rs must export governed_prompt_decision (R4: production call site); \
         file: {}",
        governance_rs.display()
    );
}

// ---------------------------------------------------------------------------
// (e) SCHEMA-LOCK: exact pins for the I-33 bump.
// ---------------------------------------------------------------------------

#[test]
fn i33_e_schema_lock() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 240,
        "SCHEMA_VERSION_PUBLIC must be >= 240 (I-33 floor: GovernancePrimitiveFired add); \
         got {SCHEMA_VERSION_PUBLIC}"
    );

    use std::fs;
    let lock_rs = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|root| {
            root.join("crates")
                .join("orchestrator")
                .join("tests")
                .join("action_type_lock.rs")
        })
        .expect("resolve action_type_lock.rs path");
    let body = fs::read_to_string(&lock_rs).expect("read action_type_lock.rs");
    // I-33 added GovernancePrimitiveFired -> TOTAL_VARIANTS must be >= 549.
    // (Subsequent I-plans bump the value; we use a floor check so i33's contract
    // stays valid after future schema bumps.)
    let total_variants_value: usize = body
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("const TOTAL_VARIANTS: usize = ") {
                trimmed
                    .trim_start_matches("const TOTAL_VARIANTS: usize = ")
                    .trim_end_matches(';')
                    .parse()
                    .ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!(
            "could not parse TOTAL_VARIANTS from action_type_lock.rs; file: {}",
            lock_rs.display()
        ));
    assert!(
        total_variants_value >= 549,
        "TOTAL_VARIANTS must be >= 549 (I-33 floor: GovernancePrimitiveFired added 1 wire tag at 549); \
         got {} in {}",
        total_variants_value,
        lock_rs.display()
    );
    assert!(
        body.contains("SsbActionType::GovernancePrimitiveFired"),
        "ALL_VARIANTS must include GovernancePrimitiveFired; file: {}",
        lock_rs.display()
    );

    // Wire-tag round-trip via public from_str.
    let at = ssb_action_type_from_str_public("governance_primitive_fired")
        .expect("from_str governance_primitive_fired must succeed");
    assert_eq!(
        at.as_sql_str(),
        "governance_primitive_fired",
        "as_sql_str round-trip must hold"
    );
}
