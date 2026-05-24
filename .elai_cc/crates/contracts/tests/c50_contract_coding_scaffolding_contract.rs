//! C-50 — Contract-Coding STRUCTURAL contract test scaffolding.
//!
//! Paper grounding: arXiv:2604.13100 (Lin, Zhao, Shi 2026 —
//! "Contract-Coding: Towards Repo-Level Generation via Structured
//! Symbolic Paradigm"). Interface map at
//! `workflow/papers/contract-coding-interface.md`.
//!
//! CITATION DISCIPLINE (load-bearing — do NOT misframe):
//!   - The paper's kernel is `K = <N, Sigma, Delta>` (Section 4.2);
//!     it is STRUCTURAL, not behavioral. The paper uses no Hoare
//!     triples, no Eiffel/JML-style pre/postconditions, no temporal
//!     logic. The C-50 surface ships structural contracts only.
//!   - The headline "47%" is the Roguelike subset of Greenfield-5
//!     (Table 1, Section 5.2) and TIES Trae. Gemini Studio beats at
//!     63%; OpenHands trails at 30%. The paper's differentiator is
//!     qualitative — failures are "strictly confined to Local Logic
//!     Errors, not architectural collapse" per Section 5.2.
//!   - Mutation primitives are deliberately {Add, Update} only
//!     (paper §4.3). Delete is excluded to avoid kernel
//!     inconsistency.
//!   - The full Generator -> Discriminator RL training pipeline
//!     (Algorithm 1, §4.4 + §4.5) is RUNPOD-PENDING. This contract
//!     fences the heuristic structural surface only.
//!
//! Contract fences (G1..G6):
//!   G1. [`StructuralContract`] carries `namespace` (`N`) +
//!       `signatures` (`Sigma`) + `state_space` (`Delta`) per
//!       paper kernel K = <N, Sigma, Delta>.
//!   G2. [`emit_contract_test`] produces a Rust-shaped string
//!       containing the target function name + signature anchor.
//!   G3. [`audit_against_contract`] returns `is_consistent = false`
//!       on a synthetic kernel violation.
//!   G4. [`ContractMutation`] has exactly 2 variants (Add + Update)
//!       per paper §4.3 closure.
//!   G5. The two new `SsbActionType::Contract*` variants round-trip
//!       through `as_sql_str` + `ssb_action_type_from_str_public`
//!       and their payloads carry the correct discriminant via
//!       `SsbPayload::action_type`.
//!   G6. SCHEMA_VERSION floor for the C-50 cascade (>= 125).

use elai_orchestrator::contract_coding::{
    audit_against_contract, emit_contract_test, scaffold_contract_test_from_signature,
    AuditResult, ContractMutation, StateConstraint, StructuralContract, TypeSignature,
    ViolationKind,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, ContractTestScaffoldedPayload,
    ContractViolationDetectedPayload, SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC,
};

// ---------------------------------------------------------------------------
// G1 — StructuralContract carries N + Sigma + Delta per paper kernel
// K = <N, Sigma, Delta> (§4.2).
// ---------------------------------------------------------------------------

#[test]
fn g1_structural_contract_has_n_sigma_delta_fields() {
    let c = StructuralContract::new(
        vec!["alpha".into(), "beta".into()],
        vec![TypeSignature::new("add", "fn add(a: i64, b: i64) -> i64", "rust")],
        vec![StateConstraint::new("totality", 0, "add is total on i64")],
    );
    // N (namespace) is populated.
    assert_eq!(c.namespace.len(), 2, "N namespace must carry 2 sections");
    assert_eq!(c.namespace[0], "alpha");
    // Sigma (signatures) is populated.
    assert_eq!(c.signatures.len(), 1, "Sigma must carry 1 signature");
    assert_eq!(c.signatures[0].function_name, "add");
    // Delta (state_space) is populated.
    assert_eq!(c.state_space.len(), 1, "Delta must carry 1 constraint");
    assert_eq!(c.state_space[0].name, "totality");
    // Kernel validates.
    c.validate().expect("well-formed kernel validates");
}

#[test]
fn g1_kernel_rejects_duplicate_namespace_sections() {
    let bad = StructuralContract::new(
        vec!["alpha".into(), "alpha".into()],
        vec![],
        vec![],
    );
    assert!(
        bad.validate().is_err(),
        "paper §4.2 requires pairwise-disjoint N sections"
    );
}

// ---------------------------------------------------------------------------
// G2 — emit_contract_test produces a Rust-shaped body. We do not
// shell out to syn; the heuristic check is "contains the target
// signature anchor + a #[test] header".
// ---------------------------------------------------------------------------

#[test]
fn g2_emit_contract_test_produces_rust_shape() {
    let c = StructuralContract::new(
        vec!["math".into()],
        vec![TypeSignature::new("add", "fn add(a: i64, b: i64) -> i64", "rust")],
        vec![StateConstraint::new("totality", 0, "add is total")],
    );
    let body = emit_contract_test(&c, "add").expect("emit");
    assert!(
        body.contains("#[test]"),
        "emitted body must contain a #[test] header"
    );
    assert!(
        body.contains("fn add(a: i64, b: i64) -> i64"),
        "emitted body must reference the target signature"
    );
    assert!(
        body.contains("Roguelike subset"),
        "emitted body must carry the paper's structural-integrity qualifier"
    );
    assert!(
        body.contains("structural"),
        "emitted body must label the kernel as STRUCTURAL (not pre/postcondition)"
    );
}

#[test]
fn g2_scaffold_from_signature_emits_kernel_anchors() {
    let sig = TypeSignature::new("compute", "fn compute(x: u32) -> u32", "rust");
    let body =
        scaffold_contract_test_from_signature("compute", &sig).expect("scaffold");
    assert!(body.contains("compute_module"));
    assert!(body.contains("compute_signature_pinned"));
    assert!(body.contains("fn compute(x: u32) -> u32"));
}

// ---------------------------------------------------------------------------
// G3 — audit_against_contract returns is_consistent=false on a
// synthetic violation. The heuristic Discriminator surfaces missing
// namespace / signature / constraint references.
// ---------------------------------------------------------------------------

#[test]
fn g3_audit_flags_missing_kernel_anchors() {
    let c = StructuralContract::new(
        vec!["alpha".into(), "beta".into()],
        vec![TypeSignature::new("add", "fn add(a: i64, b: i64) -> i64", "rust")],
        vec![StateConstraint::new("totality", 0, "add is total")],
    );
    // code does not mention `alpha` namespace section
    let bad_code = "mod beta { fn add() {} } // totality";
    let r: AuditResult = audit_against_contract(bad_code, &c);
    assert!(
        !r.is_consistent,
        "missing namespace section must flip is_consistent to false"
    );
    assert!(
        r.violations
            .iter()
            .any(|v| v.kind == ViolationKind::MissingNamespace && v.element == "alpha"),
        "missing namespace violation must be surfaced with the absent section name"
    );
}

#[test]
fn g3_audit_passes_on_complete_coverage() {
    let c = StructuralContract::new(
        vec!["alpha".into()],
        vec![TypeSignature::new("add", "fn add() -> ()", "rust")],
        vec![StateConstraint::new("totality", 0, "add is total")],
    );
    let good_code = "mod alpha { fn add() {} } // totality holds";
    let r = audit_against_contract(good_code, &c);
    assert!(r.is_consistent, "complete coverage must yield is_consistent=true");
    assert!(r.violations.is_empty());
}

// ---------------------------------------------------------------------------
// G4 — ContractMutation has exactly 2 variants per paper §4.3
// {Add, Update} closure. Delete is excluded to avoid kernel
// inconsistency.
// ---------------------------------------------------------------------------

#[test]
fn g4_mutation_alphabet_has_exactly_add_and_update() {
    let constraint = StateConstraint::new("rule_a", 0, "x");
    let add = ContractMutation::Add(constraint.clone());
    let upd = ContractMutation::Update {
        target: 0,
        new: constraint.clone(),
    };
    let tag = |m: &ContractMutation| -> &'static str {
        match m {
            ContractMutation::Add(_) => "add",
            ContractMutation::Update { .. } => "update",
        }
    };
    assert_eq!(tag(&add), "add");
    assert_eq!(tag(&upd), "update");
}

#[test]
fn g4_mutation_apply_preserves_kernel_consistency() {
    let mut c = StructuralContract::new(
        vec!["alpha".into()],
        vec![TypeSignature::new("foo", "fn foo()", "rust")],
        vec![StateConstraint::new("rule_a", 0, "x")],
    );
    let m = ContractMutation::Add(StateConstraint::new("rule_b", 0, "y"));
    m.apply_to(&mut c).expect("apply add");
    assert_eq!(c.state_space.len(), 2, "Add must extend Delta");
    c.validate().expect("kernel still consistent after Add");
}

// ---------------------------------------------------------------------------
// G5 — SSB variant round-trip for the 2 new ContractCoding rows.
// ---------------------------------------------------------------------------

#[test]
fn g5_contract_test_scaffolded_action_type_round_trips() {
    let s = SsbActionType::ContractTestScaffolded.as_sql_str();
    assert_eq!(s, "contract_test_scaffolded");
    let back = ssb_action_type_from_str_public(s).expect("from_str");
    assert_eq!(back, SsbActionType::ContractTestScaffolded);
}

#[test]
fn g5_contract_violation_detected_action_type_round_trips() {
    let s = SsbActionType::ContractViolationDetected.as_sql_str();
    assert_eq!(s, "contract_violation_detected");
    let back = ssb_action_type_from_str_public(s).expect("from_str");
    assert_eq!(back, SsbActionType::ContractViolationDetected);
}

#[test]
fn g5_contract_payloads_carry_their_action_type_discriminant() {
    let scaffolded = SsbPayload::ContractTestScaffolded(ContractTestScaffoldedPayload {
        fn_name_hash: 0xC50C_50C5_0C50_C50C,
        contract_kind_hash: 0xC0DE_C0DE_C0DE_C0DE,
        ts_ms: 1_748_044_900_000,
    });
    assert_eq!(scaffolded.action_type(), SsbActionType::ContractTestScaffolded);

    let violation = SsbPayload::ContractViolationDetected(ContractViolationDetectedPayload {
        fn_name_hash: 0xC50C_50C5_0C50_C50C,
        violation_kind_hash: 0xBEEF_BEEF_BEEF_BEEF,
        severity: 1,
        ts_ms: 1_748_044_900_000,
    });
    assert_eq!(
        violation.action_type(),
        SsbActionType::ContractViolationDetected
    );
}

#[test]
fn g5_contract_payloads_serde_round_trip_via_rmp() {
    let original = SsbPayload::ContractViolationDetected(ContractViolationDetectedPayload {
        fn_name_hash: 0x1111_2222_3333_4444,
        violation_kind_hash: 0xAAAA_BBBB_CCCC_DDDD,
        severity: 1,
        ts_ms: 1_748_044_900_000,
    });
    let bytes = rmp_serde::to_vec_named(&original).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(decoded.action_type(), original.action_type());
}

// ---------------------------------------------------------------------------
// G6 — SCHEMA_VERSION floor for the C-50 cascade. C-50 lands at
// >= 125; concurrent sibling plans may bump higher. Floor-not-
// equality so the test does not false-fail when a sibling
// has already landed.
// ---------------------------------------------------------------------------

#[test]
fn g6_schema_version_at_or_above_c50_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 125,
        "SCHEMA_VERSION must be >= 125 after C-50 cascade (got {})",
        SCHEMA_VERSION_PUBLIC
    );
}
