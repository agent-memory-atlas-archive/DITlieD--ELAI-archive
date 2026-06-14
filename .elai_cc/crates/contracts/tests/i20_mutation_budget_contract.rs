//! I-20 — budgeted mutation scheduler contract tests (Track 15 KEYSTONE).
//!
//! Coverage (no #[ignore], no stubs, no placeholder asserts):
//!
//! (a) Diff-cone selection: select_cone with no symbol-graph cache degrades to
//!     the identity set (never drops coverage, never panics).
//!
//! (b) Wilson-CI early-stop BOTH directions: pass-early when the CI lower bound
//!     clears the threshold, fail-early when the CI upper bound fails it; both
//!     stop before exhausting the cone (deterministic under a fixed input).
//!
//! (c) Cache-hit zero runs: the cone-cache summary reports zero sampled mutants
//!     and the CacheHit verdict.
//!
//! (d) Survivor-directive fields: a surviving mutant yields a SurvivorDirective
//!     carrying operator + line + a non-empty surviving-mutation description.
//!
//! (e) ID-5 threshold-from-toml: the gate threshold is READ FROM
//!     metrics/mutation_baseline.toml, NOT a hard-coded paper constant; a missing
//!     baseline is an ERROR (never a 0.80 paper fallback). A `[DESIGN]`-style
//!     literal-0.80 arm is caught: the loader refuses to invent it.
//!
//! (f) EquivalentMutantCache key extension: the I-20 4-input key folds the
//!     bound-test-set hash and is disjoint from the C-28 3-input key (old keys
//!     miss-then-rerun, never error).
//!
//! (g) Real-DB SSB landing (off-author artifact, wiring-enforcement R4): a
//!     MutationBudgetedRun payload written to a real temp SQLite SSB via the
//!     production emit fn, read back via query_by_type, and decoded; the row
//!     content is verified. NOT a struct-only / serialize-only check.
//!
//! (h) Action-type round-trip: all four mutation_budgeted_run_* sql strings
//!     round-trip via ssb_action_type_from_str_public; the MutationBudgetedRun
//!     representative arm is present in SSB_ACTION_TYPES (derived count floor).
//!
//! Spec references: docs/thefinalglance/I-plan/I-20_budgeted_mutation_scheduler.md
//! Reads (mandatory): wiring-enforcement.md R1/R4/R7/R8; CLAUDE.md ID-5 + GD-1/GD-2.

use std::path::PathBuf;
use std::sync::Arc;

use elai_orchestrator::mutation_budget::{
    compute_key_with_testset, early_stop_decision, survivor_directives, wilson_interval,
    BudgetVerdict, MutationBudget, StopDecision,
};
use elai_orchestrator::mutation_testing::{
    EquivalentMutantCache, MutantOutcome, MutantRecord, MutationResult,
};
use elai_orchestrator::deterministic_audit_gate::emit_mutation_budgeted_runs;
use elai_orchestrator::ssb::{
    decode_ssb_payload, ssb_action_type_from_str_public, MutationBudgetedRunKind, SsbActionType,
    SsbConfig, SsbPayload, SSB_ACTION_TYPES,
};

fn rec(file: &str, kind: &str, line: u32, outcome: MutantOutcome) -> MutantRecord {
    MutantRecord {
        file: PathBuf::from(file),
        line,
        mutant_kind: kind.to_owned(),
        outcome,
    }
}

// ---------------------------------------------------------------------------
// (a) Diff-cone selection — identity degrade
// ---------------------------------------------------------------------------

#[test]
fn i20_select_cone_no_cache_is_identity() {
    let files = vec![
        PathBuf::from("src/a.rs"),
        PathBuf::from("src/b.rs"),
        PathBuf::from("src/c.py"),
    ];
    let cone = MutationBudget::select_cone(None, &files);
    assert_eq!(
        cone.len(),
        3,
        "with no symbol-graph cache the cone must degrade to the identity set (never drop coverage)"
    );
}

// ---------------------------------------------------------------------------
// (b) Wilson-CI early-stop both directions
// ---------------------------------------------------------------------------

#[test]
fn i20_wilson_pass_early_stops_before_exhaustion() {
    let budget = MutationBudget::with_threshold(0.5);
    // 300 all-killed mutants; pass-early must fire well before exhausting them.
    let mutants = vec![true; 300];
    let (ci, verdict) = budget.sample_with_ci(&mutants);
    assert_eq!(verdict, BudgetVerdict::CiCleared, "all-killed must clear a 0.5 threshold");
    assert!(
        ci.n < 300,
        "pass-early must stop before exhausting all 300 mutants; sampled {}",
        ci.n
    );
    assert!(ci.lower >= 0.5, "CI lower bound must clear the threshold on pass-early");
}

#[test]
fn i20_wilson_fail_early_stops_before_exhaustion() {
    let budget = MutationBudget::with_threshold(0.8);
    // 300 all-survived mutants; fail-early must fire before exhausting them.
    let mutants = vec![false; 300];
    let (ci, verdict) = budget.sample_with_ci(&mutants);
    assert_eq!(verdict, BudgetVerdict::CiFailed, "all-survived must fail a 0.8 threshold");
    assert!(
        ci.n < 300,
        "fail-early must stop before exhausting all 300 mutants; sampled {}",
        ci.n
    );
    assert!(ci.upper < 0.8, "CI upper bound must fail the threshold on fail-early");
}

#[test]
fn i20_early_stop_decision_directions() {
    // High kill rate -> pass-early.
    let high = wilson_interval(80, 80);
    assert_eq!(early_stop_decision(&high, 0.5), StopDecision::PassEarly);
    // Zero kills over many samples -> fail-early.
    let low = wilson_interval(0, 80);
    assert_eq!(early_stop_decision(&low, 0.8), StopDecision::FailEarly);
}

// ---------------------------------------------------------------------------
// (c) Cache-hit zero runs
// ---------------------------------------------------------------------------

#[test]
fn i20_cache_hit_summary_zero_runs() {
    let budget = MutationBudget::with_threshold(0.7);
    let summary = budget.cache_hit_summary(5);
    assert_eq!(summary.verdict, BudgetVerdict::CacheHit);
    assert_eq!(summary.n_sampled, 0, "a cache hit runs ZERO mutants");
    assert_eq!(summary.n_total_cone, 5);
    assert!(summary.survivors.is_empty(), "a cache hit emits no survivors");
}

// ---------------------------------------------------------------------------
// (d) Survivor-directive fields
// ---------------------------------------------------------------------------

#[test]
fn i20_survivor_directive_carries_operator_line_mutation() {
    let result = MutationResult::from_records(vec![
        rec("src/cmp.rs", "ReplaceBinaryOp_Gt_Ge", 412, MutantOutcome::Survived),
        rec("src/cmp.rs", "ReplaceBinaryOp_Lt_Le", 99, MutantOutcome::Caught),
    ]);
    let dirs = survivor_directives(&result);
    assert_eq!(dirs.len(), 1, "exactly one survivor");
    let d = &dirs[0];
    assert_eq!(d.operator, "ReplaceBinaryOp_Gt_Ge");
    assert_eq!(d.file, "src/cmp.rs");
    assert_eq!(d.line, 412);
    assert!(
        !d.survivor_mutation.is_empty(),
        "survivor_mutation must be a non-empty actionable counterexample"
    );
    assert!(
        d.survivor_mutation.contains("412"),
        "survivor description should locate the surviving mutant; got {:?}",
        d.survivor_mutation
    );
}

// ---------------------------------------------------------------------------
// (e) ID-5 threshold-from-toml (NOT a paper constant)
// ---------------------------------------------------------------------------

#[test]
fn i20_id5_threshold_reads_from_toml_not_paper_constant() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let root = dir.path();

    // Missing baseline = ERROR. The loader REFUSES to substitute the 0.80
    // literature median (the [DESIGN] paper constant). This is the broken-arm
    // catch: a hard-coded 0.80 gate would FAIL this assertion.
    let absent = MutationBudget::load_threshold_pct(root);
    assert!(
        absent.is_err(),
        "ID-5: a missing baseline must ERROR, never fall back to a paper constant (0.80)"
    );

    // First deploy writes the MEASURED baseline; the threshold then reads from
    // the toml — a value that is NOT the paper 0.80 proves it is read, not invented.
    let measured = 0.6371;
    let wrote = MutationBudget::write_baseline_if_absent(root, measured).expect("write baseline");
    assert!(wrote, "first deploy must write the baseline");

    let t = MutationBudget::load_threshold_pct(root).expect("load threshold");
    assert!(
        (t - measured).abs() < 1e-4,
        "threshold must equal the MEASURED value read from the toml ({measured}); got {t}"
    );
    assert!(
        (t - 0.80).abs() > 1e-3,
        "threshold must NOT be the paper 0.80 literal (ID-5); got {t}"
    );

    // The baseline file actually exists on disk (expect_artifact).
    let baseline_path = root.join("metrics/mutation_baseline.toml");
    assert!(
        baseline_path.exists(),
        "metrics/mutation_baseline.toml must exist after first deploy"
    );
    let body = std::fs::read_to_string(&baseline_path).expect("read baseline");
    assert!(
        body.contains("threshold"),
        "baseline toml must carry the threshold key"
    );
}

// ---------------------------------------------------------------------------
// (f) EquivalentMutantCache key extension (bound-test-set hash folded)
// ---------------------------------------------------------------------------

#[test]
fn i20_cache_key_folds_bound_test_set_hash() {
    let three = EquivalentMutantCache::compute_key(b"fn cmp(a,b)", "op", 412);
    let four = compute_key_with_testset(b"fn cmp(a,b)", "op", 412, "testsethash_v1");
    assert_ne!(
        three, four,
        "the I-20 4-input key must be disjoint from the C-28 3-input key (old keys miss-then-rerun)"
    );

    // A different bound-test-set hash changes the key (a test-suite change
    // invalidates the cached mutant outcome).
    let four_b = compute_key_with_testset(b"fn cmp(a,b)", "op", 412, "testsethash_v2");
    assert_ne!(four, four_b, "a different test-set hash must change the cache key");

    // The extended key is still a valid blake3 hex (64 chars).
    assert_eq!(four.len(), 64, "blake3 hex key must be 64 chars");
}

// ---------------------------------------------------------------------------
// (g) Real-DB SSB landing — off-author artifact (wiring-enforcement R4)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i20_ssb_row_lands_via_production_emit() {
    use elai_core::clock::{ClockProvider, OsClock};
    use elai_orchestrator::ssb::{Ssb, SsbReader};
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir");
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let project_id = "i20-contract";
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: dir.path().join("i20-ssb.db"),
                pool_size: Some(2),
                project_id: project_id.into(),
                session_id: "i20-g".into(),
            },
            clock,
        )
        .expect("ssb open"),
    );

    // Build a budget summary with a real survivor, then drive the PRODUCTION
    // emit fn (the writer-owning gate seam) — NOT a hand-built SsbPayload.
    let result = MutationResult::from_records(vec![
        rec("src/cmp.rs", "ReplaceBinaryOp_Gt_Ge", 412, MutantOutcome::Survived),
        rec("src/cmp.rs", "ReplaceBinaryOp_Eq_Ne", 50, MutantOutcome::Caught),
    ]);
    let budget = MutationBudget::with_threshold(0.6);
    let summary = budget.summarize(&result, 3, 12);

    // Production emit (default-ON; no kill-switch set).
    emit_mutation_budgeted_runs(ssb.as_ref(), &summary).await;

    // Off-author artifact (R4): a SurvivorDirective row must have landed in the db.
    let survivor_rows = ssb
        .query_by_type(SsbActionType::MutationBudgetedRun(
            MutationBudgetedRunKind::SurvivorDirective,
        ))
        .expect("query_by_type survivor");
    assert!(
        !survivor_rows.is_empty(),
        "expected >=1 MutationBudgetedRun(SurvivorDirective) row in the db; \
         found 0 (R1: missing evidence = FAIL)"
    );

    let decoded = decode_ssb_payload(&survivor_rows[0].payload).expect("decode survivor");
    match decoded {
        SsbPayload::MutationBudgetedRun(p) => {
            assert_eq!(p.kind, MutationBudgetedRunKind::SurvivorDirective);
            assert_eq!(p.operator, "ReplaceBinaryOp_Gt_Ge", "operator must round-trip");
            assert_eq!(p.survivor_file, "src/cmp.rs");
            assert_eq!(p.survivor_line, 412);
            assert!(!p.survivor_mutation.is_empty(), "survivor_mutation must round-trip non-empty");
        }
        other => panic!("expected MutationBudgetedRun; got {other:?}"),
    }

    // The cone-verdict row (CiCleared/CiFailed depending on the score) must
    // also land with the 5 CI params. Query both verdict arms; exactly one fired.
    let cleared = ssb
        .query_by_type(SsbActionType::MutationBudgetedRun(
            MutationBudgetedRunKind::CiCleared,
        ))
        .expect("query cleared");
    let failed = ssb
        .query_by_type(SsbActionType::MutationBudgetedRun(
            MutationBudgetedRunKind::CiFailed,
        ))
        .expect("query failed");
    let verdict_rows = cleared.len() + failed.len();
    assert_eq!(
        verdict_rows, 1,
        "exactly one cone-verdict row (CiCleared|CiFailed) must land; got {verdict_rows}"
    );

    let verdict_row = cleared.first().or_else(|| failed.first()).expect("a verdict row");
    let decoded_v = decode_ssb_payload(&verdict_row.payload).expect("decode verdict");
    match decoded_v {
        SsbPayload::MutationBudgetedRun(p) => {
            assert_eq!(p.n_total_cone, 3, "n_total_cone must round-trip");
            assert!(p.ci_lower <= p.ci_upper, "CI bounds must be ordered");
            assert!((p.threshold - 0.6).abs() < 1e-6, "threshold must round-trip");
        }
        other => panic!("expected MutationBudgetedRun verdict; got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (h) Action-type round-trip + registry floor
// ---------------------------------------------------------------------------

#[test]
fn i20_action_types_round_trip() {
    let arms = [
        ("mutation_budgeted_run_ci_cleared", MutationBudgetedRunKind::CiCleared),
        ("mutation_budgeted_run_ci_failed", MutationBudgetedRunKind::CiFailed),
        (
            "mutation_budgeted_run_survivor_directive",
            MutationBudgetedRunKind::SurvivorDirective,
        ),
        ("mutation_budgeted_run_cache_hit", MutationBudgetedRunKind::CacheHit),
    ];
    for (wire, expected) in arms {
        let parsed = ssb_action_type_from_str_public(wire)
            .unwrap_or_else(|e| panic!("from_str({wire:?}) failed: {e}"));
        match parsed {
            SsbActionType::MutationBudgetedRun(kind) => {
                assert_eq!(kind, expected, "round-trip kind mismatch for {wire:?}");
                assert_eq!(
                    kind.as_sql_str(),
                    wire,
                    "as_sql_str must reproduce the wire string for {wire:?}"
                );
            }
            other => panic!("expected MutationBudgetedRun for {wire:?}; got {other:?}"),
        }
    }
}

#[test]
fn i20_registry_has_representative_arm() {
    // I-20 floor: the registry includes the MutationBudgetedRun representative.
    let has_arm = SSB_ACTION_TYPES
        .iter()
        .any(|t| matches!(t, SsbActionType::MutationBudgetedRun(_)));
    assert!(
        has_arm,
        "SSB_ACTION_TYPES must include the MutationBudgetedRun representative arm"
    );
    // Derived count floor (I-20 added 1 representative on top of I-19's 563).
    const I20_TV_FLOOR: usize = 564;
    assert!(
        SSB_ACTION_TYPES.len() >= I20_TV_FLOOR,
        "SSB_ACTION_TYPES.len() must be >= {} (I-20 floor); got {}",
        I20_TV_FLOOR,
        SSB_ACTION_TYPES.len()
    );
}
