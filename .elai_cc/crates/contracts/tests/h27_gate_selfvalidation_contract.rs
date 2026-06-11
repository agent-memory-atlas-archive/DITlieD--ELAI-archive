//! H-27 contract tests — gate self-validation, the verify-the-verifiers META gate
//! (TD-4).
//!
//! Covers (TEST-FIRST, authored against the live gate surface), over a REAL Ssb SQLite
//! db (off-agent artifact, Layer-4 / R4):
//!   (1) all 8 Track-3 gates KILL their planted mutants (each gate BLOCKS its own demo-
//!       violated fixture) and do not over-block their clean fixtures => PASS
//!       (AllGatesValid) with a real GateSelfValidation SSB row + mutation_score 100.0.
//!   (2) a deliberately-WEAKENED gate (a synthetic gate whose violation fixture is not
//!       blocked) => BLOCK (GateFailedToBlock) naming the gate + a real row.
//!   (3) an EMPTY registry => BLOCK (EmptyRegistry, R1 — NOT a vacuous pass).
//!   (4) the registry drives the REAL gate evaluate fns (not a mock): a single-gate
//!       registry over deflation / leakage / shadow_exec kills its mutant AND its clean
//!       fixture is allowed, proving the real verdicts are read.
//!   (5) SCHEMA_VERSION >= 235, GateSelfValidation in the action_type bijection.
//!
//! H-27 validates the 8 OTHER gates, NEVER itself (no infinite regress) — no self-
//! application is compiled here as a done-gate (the verifier-of-verifiers is the
//! irreducible top of the stack, routed to a frozen corpus + R8; see
//! workflow/reasoning/H-27-decisions.md).
//!
//! SCHEMA_VERSION assertions use `>=` (never an exact global pin) per the
//! BRITTLE_GLOBAL_SCHEMA_PIN gotcha. TOTAL_VARIANTS / ALL_VARIANTS presence is asserted
//! by action_type_lock.rs (compile-time exhaustiveness + count-drift) and the e45
//! contract.
//!
//! Grounding:
//!   - crates/orchestrator/src/gate_self_validation.rs (H-27 pure driver).
//!   - the 8 Track-3 gate evaluate fns (deflation/leakage/lookahead/trading_null/
//!     rerun/sim_real/result_manifest/shadow_exec).
//!   - crates/cli/src/shadow_check_cmd.rs (the W-1 done-gate template H-27 mirrors).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::{
    gate_self_validation::{
        validate_gates, validate_gates_with, GateEntry, GateId, REQUIRED_MUTATION_SCORE_PCT,
    },
    ssb::{
        ssb_action_type_from_str_public, GateSelfValidationKind, Ssb, SsbActionType, SsbConfig,
        SsbPayload, SCHEMA_VERSION_PUBLIC,
    },
    SsbReader,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn test_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, project_id: &str) -> (Arc<Ssb>, std::path::PathBuf) {
    let db_path = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path: db_path.clone(),
        pool_size: Some(2),
        project_id: project_id.into(),
        session_id: format!("{project_id}-session"),
    };
    (Arc::new(Ssb::new(cfg, test_clock()).expect("ssb")), db_path)
}

/// Read the single GateSelfValidation row emitted by `validate_gates`.
fn only_self_validation_row(ssb: &Arc<Ssb>) -> SsbPayload {
    let rows = ssb
        .query_by_type(SsbActionType::GateSelfValidation)
        .expect("query GateSelfValidation");
    assert_eq!(
        rows.len(),
        1,
        "exactly 1 GateSelfValidation SSB row (off-agent artifact, R4)"
    );
    rmp_serde::from_slice(&rows[0].payload).expect("deserialize payload")
}

// ---------------------------------------------------------------------------
// (1) all 8 gates kill their planted mutants => AllGatesValid + off-agent row
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn all_gates_kill_their_mutants_pass_and_emit_row() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h27-contract-1");

    let outcome = validate_gates(&ssb, &db_path, "h27-contract-1", 42, false)
        .await
        .expect("validate_gates must not error");

    assert_eq!(
        outcome.kind,
        GateSelfValidationKind::AllGatesValid,
        "every Track-3 gate must kill its own planted mutant (verify-the-verifiers PASS)"
    );
    assert!(outcome.is_pass());
    assert_eq!(outcome.gates_total, 8, "the frozen registry has 8 Track-3 gates");
    assert_eq!(outcome.gates_killed, 8, "all 8 mutants killed");
    assert_eq!(
        outcome.mutation_score_pct, REQUIRED_MUTATION_SCORE_PCT,
        "100% kill rate required for AllGatesValid"
    );

    if let SsbPayload::GateSelfValidation(p) = only_self_validation_row(&ssb) {
        assert_eq!(p.kind, GateSelfValidationKind::AllGatesValid);
        assert_eq!(p.gates_total, 8);
        assert_eq!(p.gates_killed, 8);
        assert_eq!(p.mutation_score_pct, 100.0);
        assert!(p.escaped_gate.is_empty(), "no escaped gate on AllGatesValid");
        assert_eq!(p.corpus_blake3.len(), 64, "blake3 hex is 64 chars");
    } else {
        panic!("payload must be GateSelfValidation");
    }
}

// ---------------------------------------------------------------------------
// (2) a deliberately-weakened gate (surviving mutant) => GateFailedToBlock
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn weakened_gate_surfaces_as_failed_to_block_and_emits_row() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h27-contract-2");

    // The full registry PLUS a synthetic weakened gate whose violation fixture is not
    // blocked => the mutant SURVIVES => GateFailedToBlock (fail-open caught, R1).
    let outcome = validate_gates(&ssb, &db_path, "h27-contract-2", 42, true)
        .await
        .expect("validate_gates");

    assert_eq!(
        outcome.kind,
        GateSelfValidationKind::GateFailedToBlock,
        "a gate that fails to block its violation (surviving mutant) must BLOCK"
    );
    assert!(!outcome.is_pass());
    assert_eq!(outcome.escaped_gate, "weakened_demo_gate");
    assert!(
        outcome.mutation_score_pct < REQUIRED_MUTATION_SCORE_PCT,
        "a surviving mutant drops the kill rate below 100%"
    );

    if let SsbPayload::GateSelfValidation(p) = only_self_validation_row(&ssb) {
        assert_eq!(p.kind, GateSelfValidationKind::GateFailedToBlock);
        assert_eq!(p.escaped_gate, "weakened_demo_gate");
        assert_eq!(p.escaped_fixture, "violation_not_blocked");
    } else {
        panic!("payload must be GateSelfValidation");
    }
}

// ---------------------------------------------------------------------------
// (3) an empty registry => EmptyRegistry BLOCK (R1, not a vacuous pass)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn empty_registry_is_fail_closed_not_vacuous_pass() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h27-contract-3");

    let outcome = validate_gates_with(&ssb, &db_path, &[], false)
        .await
        .expect("validate_gates_with");

    assert_eq!(
        outcome.kind,
        GateSelfValidationKind::EmptyRegistry,
        "a self-validation that checks ZERO gates is itself a FAIL (R1 failOnEmptyShould)"
    );
    assert!(!outcome.is_pass());
    assert_eq!(outcome.gates_total, 0);
    assert_ne!(
        outcome.mutation_score_pct, 100.0,
        "an empty should-set must NEVER emit the vacuous 100.0 pass"
    );
}

// ---------------------------------------------------------------------------
// (4) the registry drives the REAL gate evaluate fns (not a mock): a single-gate
//     registry over each of deflation / leakage / shadow_exec kills its mutant AND
//     allows its clean fixture, proving the real verdicts are read.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registry_drives_real_deflation_evaluate_fn() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h27-contract-4a");
    // Deflation has no clean fixture (math-fragile Allow path) => the load-bearing leg
    // is kill-the-mutant only. The real evaluate_deflation must Block its absent-PBO
    // mutant => AllGatesValid for this 1-gate registry.
    let registry = &[GateEntry::new(GateId::Deflation, false)];
    let outcome = validate_gates_with(&ssb, &db_path, registry, false)
        .await
        .expect("validate_gates_with");
    assert_eq!(outcome.kind, GateSelfValidationKind::AllGatesValid);
    assert_eq!(outcome.gates_total, 1);
    assert_eq!(outcome.gates_killed, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registry_drives_real_leakage_evaluate_fn() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h27-contract-4b");
    // leakage: the real evaluate_leakage must SoundBlock the fit-reaches-both mutant AND
    // Allow the train-only clean fixture (so both real verdict classes are exercised).
    let registry = &[GateEntry::new(GateId::Leakage, true)];
    let outcome = validate_gates_with(&ssb, &db_path, registry, false)
        .await
        .expect("validate_gates_with");
    assert_eq!(
        outcome.kind,
        GateSelfValidationKind::AllGatesValid,
        "leakage must block its leak mutant and allow its clean fixture"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registry_drives_real_shadow_exec_evaluate_fn() {
    let dir = TempDir::new().unwrap();
    let (ssb, db_path) = test_ssb(&dir, "h27-contract-4c");
    // shadow_exec: the real diff_shadow_impls must InvariantViolated the PnL-additivity
    // mutant AND InvariantHeld the consistent clean fixture.
    let registry = &[GateEntry::new(GateId::ShadowExec, true)];
    let outcome = validate_gates_with(&ssb, &db_path, registry, false)
        .await
        .expect("validate_gates_with");
    assert_eq!(
        outcome.kind,
        GateSelfValidationKind::AllGatesValid,
        "shadow_exec must block its PnL-additivity mutant and allow its consistent fixture"
    );
}

// ---------------------------------------------------------------------------
// (5) SSB lockstep: SCHEMA_VERSION >= 235, action_type bijection landed
// ---------------------------------------------------------------------------

#[test]
fn schema_version_and_variant_lockstep_landed() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 235,
        "SCHEMA_VERSION must be >= 235 after H-27; got {SCHEMA_VERSION_PUBLIC}"
    );

    // GateSelfValidation must round-trip through the action_type bijection.
    let at = ssb_action_type_from_str_public("gate_self_validation")
        .expect("gate_self_validation must be a known action_type");
    assert_eq!(at, SsbActionType::GateSelfValidation);
    assert_eq!(
        SsbActionType::GateSelfValidation.as_sql_str(),
        "gate_self_validation"
    );
}
